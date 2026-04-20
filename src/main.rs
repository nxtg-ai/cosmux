mod config;
mod error;
mod hooks;
mod recover;
mod state;
mod templates;
mod tmux;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use config::{resolve_pod_path, PodConfig};
use error::{CosmuxError, Result};
use hooks::{run_hooks, HookKind};
use tmux::{PodSpawner, Tmux};

#[derive(Parser)]
#[command(
    name = "cosmux",
    version,
    about = "CoS-aware tmux pod manager — declarative YAML, lifecycle hooks",
    long_about = "cosmux turns tmux sessions into declarative \"pods\" with lifecycle hooks.\n\
                  Built by NXTG-AI. Apache-2.0. https://github.com/nxtg-ai/cosmux"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, help = "Verbose logging (sets RUST_LOG=debug)")]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Spawn a pod (creates the tmux session detached)")]
    Start {
        #[arg(help = "Pod name or path to YAML config")]
        pod: String,
        #[arg(long, help = "Replace existing session if present")]
        force: bool,
        #[arg(long, help = "Attach to the session after spawn")]
        attach: bool,
    },

    #[command(about = "Kill a pod's tmux session")]
    Stop {
        #[arg(help = "Pod name (matches tmux session name)")]
        pod: String,
    },

    #[command(about = "List running tmux sessions")]
    List,

    #[command(about = "Validate a pod YAML config (no side effects)")]
    Validate {
        #[arg(help = "Pod name or path to YAML config")]
        pod: String,
    },

    #[command(about = "Print the resolved pod config (after template merge)")]
    Show {
        #[arg(help = "Pod name or path to YAML config")]
        pod: String,
    },

    #[command(about = "Print HUD state.json path + contents", alias = "hud")]
    State,

    #[command(about = "List cosmux-managed pods only (vs `list` which shows all tmux sessions)")]
    Ps,

    #[command(about = "Garbage-collect state.json: drop entries whose tmux session no longer exists")]
    Gc,

    #[command(about = "Reload a pod (stop + start). Re-reads YAML; loses claude conversation context.")]
    Reload {
        #[arg(help = "Pod name (matches tmux session name)")]
        pod: String,
        #[arg(long, help = "Attach to the session after reload")]
        attach: bool,
    },

    #[command(about = "Print shell completion script (bash | zsh | fish | powershell | elvish)")]
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },

    #[command(
        name = "_pane-recover",
        hide = true,
        about = "(internal) tmux pane-died handler"
    )]
    PaneRecover { session: String },

    #[command(
        name = "_after-detach",
        hide = true,
        about = "(internal) tmux client-detached handler"
    )]
    AfterDetach { session: String },
}

fn main() {
    let cli = Cli::parse();
    init_logging(cli.verbose);

    let result = match &cli.command {
        Commands::Start { pod, force, attach } => cmd_start(pod, *force, *attach),
        Commands::Stop { pod } => cmd_stop(pod),
        Commands::List => cmd_list(),
        Commands::Validate { pod } => cmd_validate(pod),
        Commands::Show { pod } => cmd_show(pod),
        Commands::State => cmd_state(),
        Commands::Ps => cmd_ps(),
        Commands::Gc => cmd_gc(),
        Commands::Reload { pod, attach } => cmd_reload(pod, *attach),
        Commands::Completions { shell } => cmd_completions(*shell),
        Commands::PaneRecover { session } => recover::pane_recover(session),
        Commands::AfterDetach { session } => recover::after_detach(session),
    };

    if let Err(e) = result {
        eprintln!("cosmux: {e}");
        std::process::exit(1);
    }
}

fn init_logging(verbose: bool) {
    let default_level = if verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_level))
        .format_timestamp(None)
        .format_target(false)
        .init();
}

fn load_pod(name_or_path: &str) -> Result<PodConfig> {
    let path = resolve_pod_path(name_or_path)?;
    let mut pod = PodConfig::load(&path)?;
    templates::apply_templates(&mut pod)?;
    Ok(pod)
}

fn cmd_start(name_or_path: &str, force: bool, attach: bool) -> Result<()> {
    let pod = load_pod(name_or_path)?;
    let source_path = config::resolve_pod_path(name_or_path)?;
    log::info!("starting pod '{}'", pod.name);

    run_hooks(HookKind::BeforeStart, &pod.before_start, &pod.name)?;

    let spawner = PodSpawner::new(&pod, force);
    spawner.spawn()?;

    if let Err(e) = state::record_spawn(&pod, &source_path) {
        log::warn!("state.json write failed: {e}");
    }

    run_hooks(HookKind::AfterStart, &pod.after_start, &pod.name)?;

    println!("pod '{}' is running", pod.name);
    println!("attach with: tmux attach -t {}", pod.name);

    if attach {
        run_hooks(HookKind::BeforeAttach, &pod.before_attach, &pod.name)?;
        attach_to(&pod.name)?;
    }
    Ok(())
}

fn cmd_stop(name: &str) -> Result<()> {
    if !Tmux::session_exists(name) {
        println!("pod '{name}' is not running");
        let _ = state::record_stop(name);
        return Ok(());
    }
    Tmux::kill_session(name)?;
    let _ = state::record_stop(name);
    println!("pod '{name}' stopped");
    Ok(())
}

fn cmd_state() -> Result<()> {
    let path = state::state_path();
    println!("state file: {}", path.display());
    let s = state::load()?;
    let raw = serde_json::to_string_pretty(&s)
        .map_err(|e| CosmuxError::Other(anyhow::anyhow!("state pretty: {e}")))?;
    println!("{raw}");
    Ok(())
}

fn cmd_ps() -> Result<()> {
    let s = state::load()?;
    if s.pods.is_empty() {
        println!("no cosmux pods recorded — run `cosmux start <pod>` to spawn one");
        return Ok(());
    }
    let mut alive = 0usize;
    let mut stale = 0usize;
    println!("{:<24}  {:<8}  {:<22}  source", "POD", "STATUS", "STARTED");
    for (name, pod) in &s.pods {
        let status = if Tmux::session_exists(name) {
            alive += 1;
            "alive"
        } else {
            stale += 1;
            "stale"
        };
        println!(
            "{:<24}  {:<8}  {:<22}  {}",
            name, status, pod.started_at, pod.source_path
        );
    }
    println!("\n{} alive, {} stale (run `cosmux gc` to prune stale)", alive, stale);
    Ok(())
}

fn cmd_gc() -> Result<()> {
    let mut s = state::load()?;
    let before = s.pods.len();
    s.pods.retain(|name, _| Tmux::session_exists(name));
    let removed = before - s.pods.len();
    state::save(&s)?;
    println!("gc: removed {removed} stale entries, {} pods remain", s.pods.len());
    Ok(())
}

fn cmd_completions(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let bin = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin, &mut std::io::stdout());
    Ok(())
}

fn cmd_reload(name_or_path: &str, attach: bool) -> Result<()> {
    // Reload = stop + start. Re-reads YAML so config edits take effect.
    // KILLS the running session — claude conversations in pod panes lose interactive
    // state (visible scrollback only persists if user captured it). For
    // context-preserving reload, wait for v0.5 (per-pane diff-aware respawn).
    let pod = load_pod(name_or_path)?;
    let session_name = pod.name.clone();
    if Tmux::session_exists(&session_name) {
        log::info!("reload: stopping '{}'", session_name);
        Tmux::kill_session(&session_name)?;
        let _ = state::record_stop(&session_name);
    } else {
        log::info!("reload: '{}' was not running, treating as fresh start", session_name);
    }
    cmd_start(name_or_path, false, attach)
}

fn cmd_list() -> Result<()> {
    let sessions = Tmux::list_sessions()?;
    if sessions.is_empty() {
        println!("no tmux sessions running");
        return Ok(());
    }
    println!("running tmux sessions:");
    for s in sessions {
        println!("  {s}");
    }
    Ok(())
}

fn cmd_validate(name_or_path: &str) -> Result<()> {
    let path = resolve_pod_path(name_or_path)?;
    let pod = PodConfig::load(&path)?;
    println!(
        "OK: '{}' — {} window(s), {} pane(s) total",
        pod.name,
        pod.windows.len(),
        pod.windows.iter().map(|w| w.panes.len()).sum::<usize>()
    );
    println!("source: {}", path.display());
    Ok(())
}

fn cmd_show(name_or_path: &str) -> Result<()> {
    let pod = load_pod(name_or_path)?;
    let yaml = serde_yaml::to_string(&pod)
        .map_err(|e| CosmuxError::Other(anyhow::anyhow!("serialize failed: {e}")))?;
    print!("{yaml}");
    Ok(())
}

fn attach_to(name: &str) -> Result<()> {
    use std::process::Command;
    let status = Command::new("tmux").args(["attach", "-t", name]).status()?;
    if !status.success() {
        return Err(CosmuxError::TmuxFailed {
            cmd: format!("tmux attach -t {name}"),
            code: status.code().unwrap_or(-1),
            stderr: "attach failed".into(),
        });
    }
    Ok(())
}

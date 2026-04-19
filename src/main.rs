mod config;
mod error;
mod hooks;
mod templates;
mod tmux;

use clap::{Parser, Subcommand};
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
    log::info!("starting pod '{}'", pod.name);

    run_hooks(HookKind::BeforeStart, &pod.before_start, &pod.name)?;

    let spawner = PodSpawner::new(&pod, force);
    spawner.spawn()?;

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
        return Ok(());
    }
    Tmux::kill_session(name)?;
    println!("pod '{name}' stopped");
    Ok(())
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
    let status = Command::new("tmux")
        .args(["attach", "-t", name])
        .status()?;
    if !status.success() {
        return Err(CosmuxError::TmuxFailed {
            cmd: format!("tmux attach -t {name}"),
            code: status.code().unwrap_or(-1),
            stderr: "attach failed".into(),
        });
    }
    Ok(())
}

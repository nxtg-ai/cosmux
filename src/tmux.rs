use crate::config::{expand_path, Pane, PodConfig, Window};
use crate::error::{CosmuxError, Result};
use std::path::PathBuf;
use std::process::{Command, Output};

pub struct Tmux;

impl Tmux {
    pub fn ensure_installed() -> Result<()> {
        let probe = Command::new("tmux").arg("-V").output();
        match probe {
            Ok(out) if out.status.success() => Ok(()),
            _ => Err(CosmuxError::TmuxNotFound),
        }
    }

    pub fn run(args: &[&str]) -> Result<Output> {
        log::debug!("tmux {}", args.join(" "));
        let out =
            Command::new("tmux")
                .args(args)
                .output()
                .map_err(|e| CosmuxError::TmuxFailed {
                    cmd: format!("tmux {}", args.join(" ")),
                    code: -1,
                    stderr: e.to_string(),
                })?;
        if !out.status.success() {
            return Err(CosmuxError::TmuxFailed {
                cmd: format!("tmux {}", args.join(" ")),
                code: out.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            });
        }
        Ok(out)
    }

    pub fn session_exists(name: &str) -> bool {
        Command::new("tmux")
            .args(["has-session", "-t", name])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    pub fn list_sessions() -> Result<Vec<String>> {
        let out = Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_name}"])
            .output()?;
        if !out.status.success() {
            return Ok(Vec::new());
        }
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect())
    }

    pub fn kill_session(name: &str) -> Result<()> {
        if !Self::session_exists(name) {
            return Ok(());
        }
        Self::run(&["kill-session", "-t", name])?;
        Ok(())
    }
}

pub struct PodSpawner<'a> {
    pub pod: &'a PodConfig,
    pub force: bool,
}

impl<'a> PodSpawner<'a> {
    pub fn new(pod: &'a PodConfig, force: bool) -> Self {
        Self { pod, force }
    }

    pub fn spawn(&self) -> Result<()> {
        Tmux::ensure_installed()?;

        if Tmux::session_exists(&self.pod.name) {
            if self.force {
                log::warn!("session '{}' exists — killing (force)", self.pod.name);
                Tmux::kill_session(&self.pod.name)?;
            } else {
                return Err(CosmuxError::SessionExists(self.pod.name.clone()));
            }
        }

        let pod_root = self.pod.expanded_root();
        let first_window = self
            .pod
            .windows
            .first()
            .ok_or_else(|| CosmuxError::InvalidConfig("no windows".into()))?;
        let first_pane = first_window
            .panes
            .first()
            .ok_or_else(|| CosmuxError::InvalidConfig("no panes in first window".into()))?;

        let first_cwd = resolve_cwd(first_pane, pod_root.as_ref());
        let cwd_str = first_cwd.display().to_string();

        Tmux::run(&[
            "new-session",
            "-d",
            "-s",
            &self.pod.name,
            "-n",
            &first_window.name,
            "-c",
            &cwd_str,
        ])?;

        let win_target = format!("{}:{}", self.pod.name, first_window.name);

        if let Some(cmd) = first_pane.command.as_deref().filter(|c| !c.is_empty()) {
            self.send_keys(&win_target, cmd)?;
        }

        for pane in first_window.panes.iter().skip(1) {
            let pane_cwd = resolve_cwd(pane, pod_root.as_ref());
            Tmux::run(&[
                "split-window",
                "-t",
                &win_target,
                "-c",
                &pane_cwd.display().to_string(),
            ])?;
            if let Some(cmd) = pane.command.as_deref().filter(|c| !c.is_empty()) {
                self.send_keys(&win_target, cmd)?;
            }
        }

        if first_window.panes.len() > 1 {
            Tmux::run(&["select-layout", "-t", &win_target, &first_window.layout])?;
        }

        for window in self.pod.windows.iter().skip(1) {
            self.spawn_window(window, pod_root.as_ref())?;
        }

        self.install_session_hooks()?;

        Ok(())
    }

    fn install_session_hooks(&self) -> Result<()> {
        let session = &self.pod.name;
        // pane-died hook: invoke `cosmux _pane-recover <session>` which reads state.json
        // and re-spawns the dead pane with original cwd + command.
        if !self.pod.on_pane_dead.is_empty() {
            let exe =
                std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("cosmux"));
            let cmd = format!(
                "run-shell '{} _pane-recover {} >> /tmp/cosmux-{}.log 2>&1'",
                exe.display(),
                session,
                session
            );
            // best-effort — older tmux versions may not have pane-died as a hook target
            let _ = Tmux::run(&["set-hook", "-t", session, "pane-died", &cmd]);
            // Keep the pane around so we can detect it (not auto-close)
            let _ = Tmux::run(&["set-option", "-t", session, "remain-on-exit", "on"]);
        }
        if !self.pod.after_detach.is_empty() {
            let exe =
                std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("cosmux"));
            let cmd = format!(
                "run-shell '{} _after-detach {} >> /tmp/cosmux-{}.log 2>&1'",
                exe.display(),
                session,
                session
            );
            let _ = Tmux::run(&["set-hook", "-t", session, "client-detached", &cmd]);
        }
        Ok(())
    }

    fn spawn_window(&self, window: &Window, pod_root: Option<&PathBuf>) -> Result<()> {
        let first_pane = window.panes.first().ok_or_else(|| {
            CosmuxError::InvalidConfig(format!("window '{}' has no panes", window.name))
        })?;
        let first_cwd = resolve_cwd(first_pane, pod_root);
        let target_session = format!("{}:", self.pod.name);

        Tmux::run(&[
            "new-window",
            "-t",
            &target_session,
            "-n",
            &window.name,
            "-c",
            &first_cwd.display().to_string(),
        ])?;

        let win_target = format!("{}:{}", self.pod.name, window.name);

        if let Some(cmd) = first_pane.command.as_deref().filter(|c| !c.is_empty()) {
            self.send_keys(&win_target, cmd)?;
        }

        for pane in window.panes.iter().skip(1) {
            let pane_cwd = resolve_cwd(pane, pod_root);
            Tmux::run(&[
                "split-window",
                "-t",
                &win_target,
                "-c",
                &pane_cwd.display().to_string(),
            ])?;
            if let Some(cmd) = pane.command.as_deref().filter(|c| !c.is_empty()) {
                self.send_keys(&win_target, cmd)?;
            }
        }

        if window.panes.len() > 1 {
            Tmux::run(&["select-layout", "-t", &win_target, &window.layout])?;
        }

        Ok(())
    }

    fn send_keys(&self, target: &str, command: &str) -> Result<()> {
        Tmux::run(&["send-keys", "-t", target, command, "Enter"])?;
        Ok(())
    }
}

fn resolve_cwd(pane: &Pane, pod_root: Option<&PathBuf>) -> PathBuf {
    if let Some(cwd) = pane.cwd.as_deref() {
        return expand_path(cwd);
    }
    if let Some(root) = pod_root {
        return root.clone();
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

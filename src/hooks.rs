use crate::error::{CosmuxError, Result};
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum HookKind {
    BeforeStart,
    AfterStart,
    BeforeAttach,
    AfterDetach,
    OnPaneDead,
}

impl HookKind {
    pub fn name(&self) -> &'static str {
        match self {
            HookKind::BeforeStart => "before_start",
            HookKind::AfterStart => "after_start",
            HookKind::BeforeAttach => "before_attach",
            HookKind::AfterDetach => "after_detach",
            HookKind::OnPaneDead => "on_pane_dead",
        }
    }

    pub fn fail_aborts(&self) -> bool {
        matches!(self, HookKind::BeforeStart)
    }
}

pub fn run_hooks(kind: HookKind, commands: &[String], pod_name: &str) -> Result<()> {
    if commands.is_empty() {
        return Ok(());
    }
    log::info!("[{pod_name}] running {} hook(s) for {}", commands.len(), kind.name());
    for cmd in commands {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            continue;
        }
        log::debug!("[{pod_name}] {}: {trimmed}", kind.name());
        let status = Command::new("sh").arg("-c").arg(trimmed).status();
        match status {
            Ok(s) if s.success() => continue,
            Ok(s) => {
                let msg = format!("'{trimmed}' exited with code {}", s.code().unwrap_or(-1));
                if kind.fail_aborts() {
                    return Err(CosmuxError::HookFailed { hook: kind.name().into(), reason: msg });
                }
                log::warn!("[{pod_name}] {} non-fatal failure: {msg}", kind.name());
            }
            Err(e) => {
                let msg = format!("failed to spawn '{trimmed}': {e}");
                if kind.fail_aborts() {
                    return Err(CosmuxError::HookFailed { hook: kind.name().into(), reason: msg });
                }
                log::warn!("[{pod_name}] {} spawn error: {msg}", kind.name());
            }
        }
    }
    Ok(())
}

use crate::error::{CosmuxError, Result};
use crate::hooks::{run_hooks, HookKind};
use crate::state;
use crate::tmux::Tmux;

/// Called by tmux pane-died hook. Re-spawns the dead pane in place using
/// state.json's record of the original cwd + command.
pub fn pane_recover(session: &str) -> Result<()> {
    let Some(pod_state) = state::pod(session)? else {
        log::warn!("pane-recover: no state for pod '{session}'");
        return Ok(());
    };

    // Find dead panes by querying tmux for panes where #{pane_dead} == 1.
    let out = std::process::Command::new("tmux")
        .args([
            "list-panes",
            "-t",
            session,
            "-s",
            "-F",
            "#{window_name}|#{pane_index}|#{pane_dead}",
        ])
        .output()?;
    let listing = String::from_utf8_lossy(&out.stdout);

    for line in listing.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 3 || parts[2] != "1" {
            continue;
        }
        let window_name = parts[0];
        let pane_index: usize = parts[1].parse().unwrap_or(0);
        log::info!("pane-recover: found dead pane {window_name}.{pane_index}");

        let Some(window) = pod_state.windows.iter().find(|w| w.name == window_name) else {
            continue;
        };
        // pane_index in tmux uses base-index (often 1). Map to 0-based by sorting.
        // We re-key by ordinal position within the window.
        let panes_sorted: Vec<&state::PaneState> = window.panes.iter().collect();
        // best-effort: use min(pane_index-1, 0) mapping
        let want_idx = pane_index.saturating_sub(1).min(panes_sorted.len().saturating_sub(1));
        let Some(pane_record) = panes_sorted.get(want_idx) else {
            continue;
        };

        let target = format!("{session}:{window_name}.{pane_index}");
        // respawn-pane reuses the dead pane slot
        let _ = Tmux::run(&[
            "respawn-pane",
            "-k",
            "-t",
            &target,
            "-c",
            &pane_record.cwd,
        ]);
        if !pane_record.command.is_empty() {
            let _ = Tmux::run(&["send-keys", "-t", &target, &pane_record.command, "Enter"]);
        }
    }

    if !pod_state.on_pane_dead.is_empty() {
        run_hooks(HookKind::OnPaneDead, &pod_state.on_pane_dead, session)?;
    }
    Ok(())
}

pub fn after_detach(session: &str) -> Result<()> {
    let Some(pod_state) = state::pod(session)? else {
        return Ok(());
    };
    if pod_state.after_detach.is_empty() {
        return Ok(());
    }
    run_hooks(HookKind::AfterDetach, &pod_state.after_detach, session)?;
    Ok(())
}

#[allow(dead_code)]
pub fn unknown_session(name: &str) -> Result<()> {
    Err(CosmuxError::ConfigNotFound(format!(
        "no recorded state for session '{name}' — was it spawned by cosmux?"
    )))
}

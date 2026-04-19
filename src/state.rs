use crate::config::{expand_path, PodConfig};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateFile {
    #[serde(default)]
    pub pods: BTreeMap<String, PodState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodState {
    pub status: String,
    pub started_at: String,
    pub source_path: String,
    pub windows: Vec<WindowState>,
    #[serde(default)]
    pub on_pane_dead: Vec<String>,
    #[serde(default)]
    pub after_detach: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub name: String,
    pub panes: Vec<PaneState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneState {
    pub index: usize,
    pub cwd: String,
    pub command: String,
}

pub fn state_dir() -> PathBuf {
    expand_path("~/.cosmux")
}

pub fn state_path() -> PathBuf {
    state_dir().join("state.json")
}

pub fn load() -> Result<StateFile> {
    let path = state_path();
    if !path.exists() {
        return Ok(StateFile::default());
    }
    let raw = std::fs::read_to_string(&path)?;
    let s: StateFile = serde_json::from_str(&raw)
        .map_err(|e| crate::error::CosmuxError::Other(anyhow::anyhow!("state.json parse: {e}")))?;
    Ok(s)
}

pub fn save(state: &StateFile) -> Result<()> {
    std::fs::create_dir_all(state_dir())?;
    let raw = serde_json::to_string_pretty(state)
        .map_err(|e| crate::error::CosmuxError::Other(anyhow::anyhow!("state.json serialize: {e}")))?;
    std::fs::write(state_path(), raw)?;
    Ok(())
}

pub fn record_spawn(pod: &PodConfig, source_path: &std::path::Path) -> Result<()> {
    let mut state = load()?;
    let pod_root = pod.expanded_root();
    let now = chrono_now_iso();

    let windows = pod
        .windows
        .iter()
        .map(|w| WindowState {
            name: w.name.clone(),
            panes: w
                .panes
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let cwd = match (&p.cwd, &pod_root) {
                        (Some(c), _) => expand_path(c).display().to_string(),
                        (None, Some(r)) => r.display().to_string(),
                        (None, None) => String::from("."),
                    };
                    PaneState {
                        index: i,
                        cwd,
                        command: p.command.clone().unwrap_or_default(),
                    }
                })
                .collect(),
        })
        .collect();

    let entry = PodState {
        status: "running".into(),
        started_at: now,
        source_path: source_path.display().to_string(),
        windows,
        on_pane_dead: pod.on_pane_dead.clone(),
        after_detach: pod.after_detach.clone(),
    };

    state.pods.insert(pod.name.clone(), entry);
    save(&state)
}

pub fn record_stop(name: &str) -> Result<()> {
    let mut state = load()?;
    state.pods.remove(name);
    save(&state)
}

pub fn pod(name: &str) -> Result<Option<PodState>> {
    Ok(load()?.pods.get(name).cloned())
}

fn chrono_now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_unix(secs as i64)
}

fn format_unix(secs: i64) -> String {
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;
    let (y, mo, d) = ymd_from_days(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, m, s)
}

// Howard Hinnant's civil_from_days algorithm. Input: days since 1970-01-01.
fn ymd_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

use crate::error::{CosmuxError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodConfig {
    pub name: String,

    #[serde(default)]
    pub root: Option<String>,

    #[serde(default)]
    pub template: Option<String>,

    #[serde(default)]
    pub before_start: Vec<String>,

    #[serde(default)]
    pub after_start: Vec<String>,

    #[serde(default)]
    pub before_attach: Vec<String>,

    #[serde(default)]
    pub after_detach: Vec<String>,

    #[serde(default)]
    pub on_pane_dead: Vec<String>,

    #[serde(default)]
    pub windows: Vec<Window>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub name: String,

    #[serde(default = "default_layout")]
    pub layout: String,

    #[serde(default)]
    pub panes: Vec<Pane>,
}

fn default_layout() -> String {
    "tiled".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pane {
    #[serde(default)]
    pub cwd: Option<String>,

    #[serde(default)]
    pub command: Option<String>,

    #[serde(default)]
    pub template: Option<String>,
}

impl PodConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let path_str = path_ref.display().to_string();
        if !path_ref.exists() {
            return Err(CosmuxError::ConfigNotFound(path_str));
        }
        let raw = std::fs::read_to_string(path_ref)?;
        let cfg: PodConfig = serde_yaml::from_str(&raw)
            .map_err(|e| CosmuxError::InvalidYaml { path: path_str, source: e })?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(CosmuxError::InvalidConfig("pod name is required".into()));
        }
        if self.windows.is_empty() {
            return Err(CosmuxError::InvalidConfig(format!(
                "pod '{}' must declare at least one window",
                self.name
            )));
        }
        for w in &self.windows {
            if w.name.is_empty() {
                return Err(CosmuxError::InvalidConfig(format!(
                    "pod '{}' has a window with no name",
                    self.name
                )));
            }
            if w.panes.is_empty() {
                return Err(CosmuxError::InvalidConfig(format!(
                    "window '{}/{}' must declare at least one pane",
                    self.name, w.name
                )));
            }
        }
        Ok(())
    }

    pub fn expanded_root(&self) -> Option<PathBuf> {
        self.root.as_deref().map(expand_path)
    }
}

pub fn expand_path<S: AsRef<str>>(s: S) -> PathBuf {
    let expanded = shellexpand::tilde(s.as_ref()).to_string();
    PathBuf::from(expanded)
}

pub fn config_search_paths(name: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = dirs_home() {
        paths.push(home.join("ASIF").join("infra").join("tmux").join(format!("{name}.yaml")));
        paths.push(home.join(".config").join("cosmux").join("pods").join(format!("{name}.yaml")));
    }
    paths.push(PathBuf::from(format!("./{name}.yaml")));
    paths
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub fn resolve_pod_path(name_or_path: &str) -> Result<PathBuf> {
    let direct = PathBuf::from(name_or_path);
    if direct.exists() {
        return Ok(direct);
    }
    for candidate in config_search_paths(name_or_path) {
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(CosmuxError::ConfigNotFound(format!(
        "no pod config found for '{name_or_path}' (searched ~/ASIF/infra/tmux/, ~/.config/cosmux/pods/, ./)"
    )))
}

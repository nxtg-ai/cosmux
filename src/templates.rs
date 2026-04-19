use crate::config::{Pane, PodConfig};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaneTemplate {
    #[serde(default)]
    pub default_command: Option<String>,
    #[serde(default)]
    pub on_pane_dead: Vec<String>,
}

pub fn template_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| {
        PathBuf::from(h)
            .join(".config")
            .join("cosmux")
            .join("templates")
    })
}

pub fn load_template(name: &str) -> Result<Option<PaneTemplate>> {
    let Some(dir) = template_dir() else {
        return Ok(None);
    };
    let path = dir.join(format!("{name}.yaml"));
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)?;
    let tpl: PaneTemplate =
        serde_yaml::from_str(&raw).map_err(|e| crate::error::CosmuxError::InvalidYaml {
            path: path.display().to_string(),
            source: e,
        })?;
    Ok(Some(tpl))
}

pub fn apply_templates(pod: &mut PodConfig) -> Result<()> {
    let pod_template = pod.template.clone();
    for window in &mut pod.windows {
        for pane in &mut window.panes {
            apply_pane_template(pane, pod_template.as_deref())?;
        }
    }
    Ok(())
}

fn apply_pane_template(pane: &mut Pane, pod_template_name: Option<&str>) -> Result<()> {
    let template_name = pane.template.as_deref().or(pod_template_name);
    let Some(name) = template_name else {
        return Ok(());
    };
    let Some(tpl) = load_template(name)? else {
        log::warn!("template '{name}' referenced but not found in ~/.config/cosmux/templates/");
        return Ok(());
    };
    if pane.command.is_none() {
        pane.command = tpl.default_command;
    }
    Ok(())
}

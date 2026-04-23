use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{GrinddError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrinddConfig {
    pub state_root: PathBuf,
    pub log_level: String,
    pub cgroup_root: PathBuf,
    pub bridge_name: String,
}

impl Default for GrinddConfig {
    fn default() -> Self {
        Self {
            state_root: PathBuf::from("/var/lib/grindd"),
            log_level: "info".to_string(),
            cgroup_root: PathBuf::from("/sys/fs/cgroup"),
            bridge_name: "grindd0".to_string(),
        }
    }
}

impl GrinddConfig {
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        match config_path {
            Some(path) => Self::from_file(path),
            None => Ok(Self::default()),
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        serde_json::from_str::<Self>(&raw)
            .map_err(|err| GrinddError::Config(format!("{}: {err}", path.display())))
    }
}

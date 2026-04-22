use std::path::PathBuf;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub root_dir: PathBuf,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("/var/lib/grindd"),
        }
    }
}

pub fn ensure_root_layout(config: &DaemonConfig) -> Result<()> {
    std::fs::create_dir_all(config.root_dir.join("containers"))?;
    std::fs::create_dir_all(config.root_dir.join("images"))?;
    std::fs::create_dir_all(config.root_dir.join("runtime"))?;
    Ok(())
}
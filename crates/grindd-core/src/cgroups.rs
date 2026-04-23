use std::fs;
use std::path::{Path, PathBuf};

use crate::{GrinddError, Result};

#[derive(Debug, Clone, Default)]
pub struct CgroupLimits {
    pub memory_max: Option<u64>,
    pub cpu_max_quota: Option<u64>,
    pub cpu_max_period: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CgroupV2Manager {
    pub root: PathBuf,
}

impl Default for CgroupV2Manager {
    fn default() -> Self {
        Self {
            root: PathBuf::from("/sys/fs/cgroup"),
        }
    }
}

impl CgroupV2Manager {
    pub fn create_group(&self, name: &str) -> Result<PathBuf> {
        let path = self.root.join(name);
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    pub fn apply_limits(&self, group: &Path, limits: &CgroupLimits) -> Result<()> {
        if let Some(memory) = limits.memory_max {
            fs::write(group.join("memory.max"), memory.to_string())
                .map_err(|e| GrinddError::Cgroup(format!("write memory.max failed: {e}")))?;
        }

        if let Some(quota) = limits.cpu_max_quota {
            let period = limits.cpu_max_period.unwrap_or(100000);
            fs::write(group.join("cpu.max"), format!("{quota} {period}"))
                .map_err(|e| GrinddError::Cgroup(format!("write cpu.max failed: {e}")))?;
        }

        Ok(())
    }

    pub fn attach_pid(&self, group: &Path, pid: u32) -> Result<()> {
        fs::write(group.join("cgroup.procs"), pid.to_string())
            .map_err(|e| GrinddError::Cgroup(format!("attach pid failed: {e}")))
    }
}

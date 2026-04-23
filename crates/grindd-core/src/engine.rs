use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::buildsys::{default_cache_path, execute_build, parse_build_file};
use crate::cgroups::{CgroupLimits, CgroupV2Manager};
use crate::daemon::{ContainerRecord, Daemon, append_container_log};
use crate::image::{ImageMetadata, load_image_metadata, load_tar_image};
use crate::inspect::{ExplainReport, build_explain_report};
use crate::network::{NetworkPlan, setup_network};
use crate::runtime::{NamespaceSpec, ProcessSpec, run_process};
use crate::storage::prepare_overlay_layout;
#[cfg(target_os = "linux")]
use crate::storage::mount_overlay;
use crate::{GrinddError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRequest {
    pub id: String,
    pub image: String,
    pub command: Vec<String>,
    pub memory_max: Option<u64>,
    pub cpu_quota: Option<u64>,
}

pub struct Engine {
    pub daemon: Daemon,
    pub cgroups: CgroupV2Manager,
}

impl Engine {
    pub fn new(state_root: PathBuf) -> Self {
        Self {
            daemon: Daemon::new(state_root),
            cgroups: CgroupV2Manager::default(),
        }
    }

    pub fn bootstrap(&self) -> Result<()> {
        self.daemon.ensure_layout()
    }

    pub fn load_image(&self, name: &str, tar_path: &Path) -> Result<ImageMetadata> {
        load_tar_image(name, tar_path, &self.daemon.state_root)
    }

    pub fn run_container(&self, req: &RunRequest) -> Result<i32> {
        let image = load_image_metadata(&self.daemon.state_root, &req.image)?;
        let overlay = prepare_overlay_layout(&self.daemon.state_root, &req.id, &image.extracted_root)?;

        #[cfg(target_os = "linux")]
        mount_overlay(&overlay)?;

        let mut state = self.daemon.load_state()?;
        state.containers.insert(
            req.id.clone(),
            ContainerRecord {
                id: req.id.clone(),
                image: req.image.clone(),
                command: req.command.clone(),
                pid: None,
                state: "created".to_string(),
            },
        );
        self.daemon.save_state(&state)?;

        let ns = NamespaceSpec {
            pid: true,
            uts: true,
            mount: true,
            net: false,
            hostname: Some(format!("grindd-{}", req.id)),
            mount_proc: true,
        };

        let spec = ProcessSpec {
            argv: req.command.clone(),
            rootfs: Some(overlay.merged.clone()),
            namespaces: ns,
        };

        append_container_log(&self.daemon.state_root, &req.id, "starting container process")?;

        let code = run_process(&spec)?;

        if let Some(record) = state.containers.get_mut(&req.id) {
            record.state = "exited".to_string();
        }
        self.daemon.save_state(&state)?;

        append_container_log(
            &self.daemon.state_root,
            &req.id,
            &format!("container exited with code {code}"),
        )?;

        Ok(code)
    }

    pub fn apply_limits(&self, id: &str, pid: u32, memory: Option<u64>, cpu_quota: Option<u64>) -> Result<()> {
        let group = self.cgroups.create_group(&format!("grindd/{id}"))?;
        let limits = CgroupLimits {
            memory_max: memory,
            cpu_max_quota: cpu_quota,
            cpu_max_period: Some(100000),
        };
        self.cgroups.apply_limits(&group, &limits)?;
        self.cgroups.attach_pid(&group, pid)
    }

    pub fn setup_default_network(&self, id: &str) -> Result<()> {
        let plan = NetworkPlan {
            bridge_name: "grindd0".to_string(),
            bridge_cidr: "10.0.0.1/24".to_string(),
            veth_host: format!("vethh-{id}"),
            veth_container: format!("vethc-{id}"),
            container_ns: format!("ns-{id}"),
            container_ip: "10.0.0.2/24".to_string(),
        };
        setup_network(&plan)
    }

    pub fn list_containers(&self) -> Result<Vec<ContainerRecord>> {
        let state = self.daemon.load_state()?;
        Ok(state.containers.into_values().collect())
    }

    pub fn remove_container(&self, id: &str) -> Result<()> {
        let mut state = self.daemon.load_state()?;
        state.containers.remove(id);
        self.daemon.save_state(&state)?;
        Ok(())
    }

    pub fn container_logs(&self, id: &str) -> Result<String> {
        let path = self.daemon.state_root.join("logs").join(format!("{id}.log"));
        if !path.exists() {
            return Err(GrinddError::Daemon(format!("no logs for container {id}")));
        }
        Ok(std::fs::read_to_string(path)?)
    }

    pub fn inspect(&self, id: &str) -> Result<ContainerRecord> {
        let state = self.daemon.load_state()?;
        state
            .containers
            .get(id)
            .cloned()
            .ok_or_else(|| GrinddError::Daemon(format!("container {id} not found")))
    }

    pub fn explain(&self, id: &str) -> ExplainReport {
        build_explain_report(id)
    }

    pub fn build_from_file(&self, context: &Path, build_file: &Path) -> Result<Vec<String>> {
        let plan = parse_build_file(build_file)?;
        let cache_path = default_cache_path(&self.daemon.state_root);
        execute_build(&plan, context, &cache_path)
    }
}

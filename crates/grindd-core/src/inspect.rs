use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainStep {
    pub phase: u8,
    pub subsystem: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainReport {
    pub container_id: String,
    pub steps: Vec<ExplainStep>,
}

pub fn build_explain_report(container_id: &str) -> ExplainReport {
    ExplainReport {
        container_id: container_id.to_string(),
        steps: vec![
            ExplainStep {
                phase: 2,
                subsystem: "runtime".to_string(),
                detail: "PID/UTS/Mount namespace plan prepared".to_string(),
            },
            ExplainStep {
                phase: 3,
                subsystem: "rootfs".to_string(),
                detail: "pivot_root/chroot workflow prepared".to_string(),
            },
            ExplainStep {
                phase: 5,
                subsystem: "cgroups".to_string(),
                detail: "memory.max and cpu.max will be written".to_string(),
            },
            ExplainStep {
                phase: 7,
                subsystem: "storage".to_string(),
                detail: "overlayfs lower/upper/work/merged directories configured".to_string(),
            },
            ExplainStep {
                phase: 9,
                subsystem: "network".to_string(),
                detail: "bridge, veth, netns and NAT configuration planned".to_string(),
            },
        ],
    }
}

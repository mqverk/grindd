#[cfg(target_os = "linux")]
use std::process::Command;

use crate::{GrinddError, Result};

#[derive(Debug, Clone)]
pub struct NetworkPlan {
    pub bridge_name: String,
    pub bridge_cidr: String,
    pub veth_host: String,
    pub veth_container: String,
    pub container_ns: String,
    pub container_ip: String,
}

pub fn setup_network(plan: &NetworkPlan) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        run_ip(["link", "add", &plan.bridge_name, "type", "bridge"])?;
        run_ip(["addr", "add", &plan.bridge_cidr, "dev", &plan.bridge_name])?;
        run_ip(["link", "set", &plan.bridge_name, "up"])?;

        run_ip([
            "link",
            "add",
            &plan.veth_host,
            "type",
            "veth",
            "peer",
            "name",
            &plan.veth_container,
        ])?;
        run_ip(["link", "set", &plan.veth_host, "master", &plan.bridge_name])?;
        run_ip(["link", "set", &plan.veth_host, "up"])?;
        run_ip(["link", "set", &plan.veth_container, "netns", &plan.container_ns])?;

        let ns_exec = |args: &[&str]| -> Result<()> {
            let status = Command::new("ip")
                .args(["netns", "exec", &plan.container_ns])
                .args(args)
                .status()?;
            if !status.success() {
                return Err(GrinddError::Network(format!("ip netns exec failed: {args:?}")));
            }
            Ok(())
        };

        ns_exec(&["ip", "link", "set", "lo", "up"])?;
        ns_exec(&["ip", "link", "set", &plan.veth_container, "up"])?;
        ns_exec(&["ip", "addr", "add", &plan.container_ip, "dev", &plan.veth_container])?;

        let status = Command::new("sh")
            .args([
                "-c",
                "sysctl -w net.ipv4.ip_forward=1 >/dev/null && iptables -t nat -C POSTROUTING -s 10.0.0.0/24 ! -o grindd0 -j MASQUERADE 2>/dev/null || iptables -t nat -A POSTROUTING -s 10.0.0.0/24 ! -o grindd0 -j MASQUERADE",
            ])
            .status()?;
        if !status.success() {
            return Err(GrinddError::Network("failed to configure NAT".to_string()));
        }

        return Ok(());
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = plan;
        Err(GrinddError::Unsupported("network namespace requires Linux".to_string()))
    }
}

pub fn teardown_network(plan: &NetworkPlan) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let _ = run_ip(["link", "del", &plan.bridge_name]);
        let _ = run_ip(["netns", "del", &plan.container_ns]);
        return Ok(());
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = plan;
        Err(GrinddError::Unsupported("network namespace requires Linux".to_string()))
    }
}

#[cfg(target_os = "linux")]
fn run_ip<const N: usize>(args: [&str; N]) -> Result<()> {
    let status = Command::new("ip").args(args).status()?;
    if !status.success() {
        return Err(GrinddError::Network(format!("ip command failed: {args:?}")));
    }
    Ok(())
}

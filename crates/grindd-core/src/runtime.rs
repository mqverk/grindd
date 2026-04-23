use std::path::PathBuf;
use std::process::Command;

use crate::{GrinddError, Result};

#[derive(Debug, Clone, Default)]
pub struct NamespaceSpec {
    pub pid: bool,
    pub uts: bool,
    pub mount: bool,
    pub net: bool,
    pub hostname: Option<String>,
    pub mount_proc: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessSpec {
    pub argv: Vec<String>,
    pub rootfs: Option<PathBuf>,
    pub namespaces: NamespaceSpec,
}

pub fn run_process(spec: &ProcessSpec) -> Result<i32> {
    if spec.argv.is_empty() {
        return Err(GrinddError::Runtime("empty command".to_string()));
    }

    #[cfg(target_os = "linux")]
    {
        return run_process_linux(spec);
    }

    #[cfg(not(target_os = "linux"))]
    {
        let mut cmd = Command::new(&spec.argv[0]);
        if spec.argv.len() > 1 {
            cmd.args(&spec.argv[1..]);
        }
        let status = cmd.status()?;
        return Ok(status.code().unwrap_or(1));
    }
}

#[cfg(target_os = "linux")]
fn run_process_linux(spec: &ProcessSpec) -> Result<i32> {
    use std::ffi::CString;
    use std::os::unix::process::CommandExt;

    use crate::rootfs::{RootfsPlan, apply_rootfs};

    let mut cmd = Command::new(&spec.argv[0]);
    if spec.argv.len() > 1 {
        cmd.args(&spec.argv[1..]);
    }

    let ns = spec.namespaces.clone();
    let rootfs = spec.rootfs.clone();

    unsafe {
        cmd.pre_exec(move || {
            let mut flags: libc::c_int = 0;
            if ns.pid {
                flags |= libc::CLONE_NEWPID;
            }
            if ns.uts {
                flags |= libc::CLONE_NEWUTS;
            }
            if ns.mount {
                flags |= libc::CLONE_NEWNS;
            }
            if ns.net {
                flags |= libc::CLONE_NEWNET;
            }
            if flags != 0 && libc::unshare(flags) != 0 {
                return Err(std::io::Error::last_os_error());
            }

            if let Some(hostname) = &ns.hostname {
                let c = CString::new(hostname.as_str())
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "hostname"))?;
                if libc::sethostname(c.as_ptr(), hostname.len()) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }

            if let Some(root) = &rootfs {
                let plan = RootfsPlan {
                    root: root.clone(),
                    use_pivot_root: true,
                    bind_mounts: Vec::new(),
                };
                apply_rootfs(&plan).map_err(std::io::Error::other)?;
            }

            if ns.mount_proc {
                let _ = std::fs::create_dir_all("/proc");
                let src = CString::new("proc").expect("proc CString must be valid");
                let dst = CString::new("/proc").expect("/proc CString must be valid");
                let typ = CString::new("proc").expect("proc CString must be valid");
                if libc::mount(src.as_ptr(), dst.as_ptr(), typ.as_ptr(), 0, std::ptr::null()) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }

            Ok(())
        });
    }

    let status = cmd.status()?;
    Ok(status.code().unwrap_or(1))
}

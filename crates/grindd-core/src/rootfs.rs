#[cfg(target_os = "linux")]
use std::ffi::CString;
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::path::Path;

use crate::{GrinddError, Result};

#[derive(Debug, Clone)]
pub struct BindMount {
    pub source: PathBuf,
    pub target: PathBuf,
    pub read_only: bool,
}

#[derive(Debug, Clone)]
pub struct RootfsPlan {
    pub root: PathBuf,
    pub use_pivot_root: bool,
    pub bind_mounts: Vec<BindMount>,
}

pub fn apply_rootfs(plan: &RootfsPlan) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        return apply_rootfs_linux(plan);
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = plan;
        Err(GrinddError::Unsupported("rootfs requires Linux".to_string()))
    }
}

#[cfg(target_os = "linux")]
fn apply_rootfs_linux(plan: &RootfsPlan) -> Result<()> {
    for bind in &plan.bind_mounts {
        bind_mount_into_root(&plan.root, bind)?;
    }

    if plan.use_pivot_root {
        pivot_root_into(&plan.root)
    } else {
        chroot_into(&plan.root)
    }
}

#[cfg(target_os = "linux")]
fn bind_mount_into_root(root: &Path, bind: &BindMount) -> Result<()> {
    let relative_target = bind
        .target
        .strip_prefix("/")
        .unwrap_or(bind.target.as_path());
    let destination = root.join(relative_target);
    std::fs::create_dir_all(&destination)?;

    let src = CString::new(bind.source.to_string_lossy().as_bytes())
        .map_err(|_| GrinddError::Runtime("invalid bind source path".to_string()))?;
    let dst = CString::new(destination.to_string_lossy().as_bytes())
        .map_err(|_| GrinddError::Runtime("invalid bind target path".to_string()))?;

    let flags = libc::MS_BIND | libc::MS_REC;
    let rc = unsafe { libc::mount(src.as_ptr(), dst.as_ptr(), std::ptr::null(), flags as _, std::ptr::null()) };
    if rc != 0 {
        return Err(GrinddError::Runtime(format!(
            "bind mount {} -> {} failed: {}",
            bind.source.display(),
            bind.target.display(),
            std::io::Error::last_os_error()
        )));
    }

    if bind.read_only {
        let ro_flags = (libc::MS_BIND | libc::MS_REMOUNT | libc::MS_RDONLY) as _;
        let remount_rc = unsafe { libc::mount(std::ptr::null(), dst.as_ptr(), std::ptr::null(), ro_flags, std::ptr::null()) };
        if remount_rc != 0 {
            return Err(GrinddError::Runtime(format!(
                "read-only remount failed for {}: {}",
                bind.target.display(),
                std::io::Error::last_os_error()
            )));
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn chroot_into(root: &Path) -> Result<()> {
    let c_root = CString::new(root.to_string_lossy().as_bytes())
        .map_err(|_| GrinddError::Runtime("invalid rootfs path".to_string()))?;
    if unsafe { libc::chroot(c_root.as_ptr()) } != 0 {
        return Err(GrinddError::Runtime(format!(
            "chroot failed: {}",
            std::io::Error::last_os_error()
        )));
    }
    let slash = CString::new("/").expect("static slash path is valid CString");
    if unsafe { libc::chdir(slash.as_ptr()) } != 0 {
        return Err(GrinddError::Runtime(format!(
            "chdir after chroot failed: {}",
            std::io::Error::last_os_error()
        )));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn pivot_root_into(root: &Path) -> Result<()> {
    let root_str = root.to_string_lossy().to_string();
    let old_root = root.join(".pivot_old");
    std::fs::create_dir_all(&old_root)?;

    let c_root = CString::new(root_str.as_bytes())
        .map_err(|_| GrinddError::Runtime("invalid root path for pivot_root".to_string()))?;
    let c_old_root = CString::new(old_root.to_string_lossy().as_bytes())
        .map_err(|_| GrinddError::Runtime("invalid old root path for pivot_root".to_string()))?;

    let bind_rc = unsafe {
        libc::mount(
            c_root.as_ptr(),
            c_root.as_ptr(),
            std::ptr::null(),
            (libc::MS_BIND | libc::MS_REC) as _,
            std::ptr::null(),
        )
    };
    if bind_rc != 0 {
        return Err(GrinddError::Runtime(format!(
            "bind mount before pivot_root failed: {}",
            std::io::Error::last_os_error()
        )));
    }

    let pivot_rc = unsafe {
        libc::syscall(
            libc::SYS_pivot_root,
            c_root.as_ptr(),
            c_old_root.as_ptr(),
        )
    };
    if pivot_rc != 0 {
        return chroot_into(root);
    }

    let slash = CString::new("/").expect("static slash path is valid CString");
    if unsafe { libc::chdir(slash.as_ptr()) } != 0 {
        return Err(GrinddError::Runtime(format!(
            "chdir after pivot_root failed: {}",
            std::io::Error::last_os_error()
        )));
    }

    let old_root_path = CString::new("/.pivot_old").expect("static path is valid CString");
    let detach_rc = unsafe {
        libc::umount2(old_root_path.as_ptr(), libc::MNT_DETACH)
    };
    if detach_rc != 0 {
        return Err(GrinddError::Runtime(format!(
            "umount old root failed: {}",
            std::io::Error::last_os_error()
        )));
    }
    let _ = std::fs::remove_dir_all("/.pivot_old");

    Ok(())
}

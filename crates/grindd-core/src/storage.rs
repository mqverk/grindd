use std::fs;
use std::path::{Path, PathBuf};

use crate::{GrinddError, Result};

#[derive(Debug, Clone)]
pub struct OverlayLayout {
    pub lowerdir: PathBuf,
    pub upperdir: PathBuf,
    pub workdir: PathBuf,
    pub merged: PathBuf,
}

pub fn prepare_overlay_layout(base: &Path, container_id: &str, image_rootfs: &Path) -> Result<OverlayLayout> {
    let root = base.join("containers").join(container_id).join("overlay");
    let upperdir = root.join("upper");
    let workdir = root.join("work");
    let merged = root.join("merged");

    fs::create_dir_all(&upperdir)?;
    fs::create_dir_all(&workdir)?;
    fs::create_dir_all(&merged)?;

    Ok(OverlayLayout {
        lowerdir: image_rootfs.to_path_buf(),
        upperdir,
        workdir,
        merged,
    })
}

pub fn mount_overlay(layout: &OverlayLayout) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let options = format!(
            "lowerdir={},upperdir={},workdir={}",
            layout.lowerdir.display(),
            layout.upperdir.display(),
            layout.workdir.display()
        );

        let status = std::process::Command::new("mount")
            .args(["-t", "overlay", "overlay", "-o", &options])
            .arg(&layout.merged)
            .status()?;

        if !status.success() {
            return Err(GrinddError::Runtime("overlayfs mount failed".to_string()));
        }
        return Ok(());
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = layout;
        Err(GrinddError::Unsupported("overlayfs requires Linux".to_string()))
    }
}

pub fn unmount_overlay(layout: &OverlayLayout) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("umount").arg(&layout.merged).status()?;
        if !status.success() {
            return Err(GrinddError::Runtime("overlayfs unmount failed".to_string()));
        }
        return Ok(());
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = layout;
        Err(GrinddError::Unsupported("overlayfs requires Linux".to_string()))
    }
}

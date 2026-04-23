use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{GrinddError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub name: String,
    pub digest: String,
    pub source_tar: PathBuf,
    pub extracted_root: PathBuf,
}

pub fn load_tar_image(name: &str, tar_path: &Path, store_root: &Path) -> Result<ImageMetadata> {
    let image_root = store_root.join("images").join(name);
    let rootfs = image_root.join("rootfs");
    fs::create_dir_all(&rootfs)?;

    let mut archive = tar::Archive::new(fs::File::open(tar_path)?);
    archive.unpack(&rootfs)?;

    let digest = sha256_file(tar_path)?;
    let metadata = ImageMetadata {
        name: name.to_string(),
        digest,
        source_tar: tar_path.to_path_buf(),
        extracted_root: rootfs,
    };

    let metadata_path = image_root.join("metadata.json");
    let payload = serde_json::to_vec_pretty(&metadata)
        .map_err(|e| GrinddError::Image(format!("serialize metadata failed: {e}")))?;
    fs::write(metadata_path, payload)?;

    Ok(metadata)
}

pub fn load_image_metadata(store_root: &Path, name: &str) -> Result<ImageMetadata> {
    let path = store_root.join("images").join(name).join("metadata.json");
    let data = fs::read(path)?;
    serde_json::from_slice(&data)
        .map_err(|e| GrinddError::Image(format!("parse metadata failed: {e}")))
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

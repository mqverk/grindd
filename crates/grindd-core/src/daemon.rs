use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{GrinddError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerRecord {
    pub id: String,
    pub image: String,
    pub command: Vec<String>,
    pub pid: Option<u32>,
    pub state: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DaemonState {
    pub containers: HashMap<String, ContainerRecord>,
}

pub struct Daemon {
    pub state_root: PathBuf,
}

impl Daemon {
    pub fn new(state_root: PathBuf) -> Self {
        Self { state_root }
    }

    pub fn ensure_layout(&self) -> Result<()> {
        fs::create_dir_all(self.state_root.join("containers"))?;
        fs::create_dir_all(self.state_root.join("images"))?;
        fs::create_dir_all(self.state_root.join("logs"))?;
        fs::create_dir_all(self.state_root.join("run"))?;
        Ok(())
    }

    pub fn state_file(&self) -> PathBuf {
        self.state_root.join("daemon-state.json")
    }

    pub fn load_state(&self) -> Result<DaemonState> {
        let path = self.state_file();
        if !path.exists() {
            return Ok(DaemonState::default());
        }
        let payload = fs::read(path)?;
        serde_json::from_slice(&payload)
            .map_err(|e| GrinddError::Daemon(format!("parse daemon state failed: {e}")))
    }

    pub fn save_state(&self, state: &DaemonState) -> Result<()> {
        let payload = serde_json::to_vec_pretty(state)
            .map_err(|e| GrinddError::Daemon(format!("serialize daemon state failed: {e}")))?;
        fs::write(self.state_file(), payload)?;
        Ok(())
    }

    pub fn socket_path(&self) -> PathBuf {
        self.state_root.join("run").join("grindd.sock")
    }

    pub fn serve_once(&self) -> Result<()> {
        #[cfg(unix)]
        {
            use std::io::{Read, Write};
            use std::os::unix::net::UnixListener;

            let socket = self.socket_path();
            if socket.exists() {
                fs::remove_file(&socket)?;
            }
            let listener = UnixListener::bind(&socket)?;
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 512];
                let n = stream.read(&mut buf)?;
                let req = std::str::from_utf8(&buf[..n]).unwrap_or("{}");
                let response = format!("{{\"ok\":true,\"echo\":{req:?}}}");
                stream.write_all(response.as_bytes())?;
            }
            return Ok(());
        }

        #[cfg(not(unix))]
        {
            Err(GrinddError::Unsupported(
                "unix socket API requires unix platform".to_string(),
            ))
        }
    }
}

pub fn append_container_log(state_root: &Path, container_id: &str, line: &str) -> Result<()> {
    let log_path = state_root.join("logs").join(format!("{container_id}.log"));
    let mut existing = String::new();
    if log_path.exists() {
        existing = fs::read_to_string(&log_path)?;
    }
    existing.push_str(line);
    existing.push('\n');
    fs::write(log_path, existing)?;
    Ok(())
}

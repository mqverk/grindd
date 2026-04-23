use crate::{GrinddError, Result};

pub fn run_init_reaper_loop() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        loop {
            let mut status: libc::c_int = 0;
            let pid = unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG) };
            if pid <= 0 {
                break;
            }
        }
        return Ok(());
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err(GrinddError::Unsupported("reaper loop requires Linux".to_string()))
    }
}

pub fn forward_signal(pid: i32, signal: i32) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let rc = unsafe { libc::kill(pid, signal) };
        if rc != 0 {
            return Err(GrinddError::Runtime(format!(
                "failed to forward signal {} to {}: {}",
                signal,
                pid,
                std::io::Error::last_os_error()
            )));
        }
        return Ok(());
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = (pid, signal);
        Err(GrinddError::Unsupported("signal forwarding requires Linux".to_string()))
    }
}

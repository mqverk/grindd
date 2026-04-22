use std::process::Command;

use crate::error::{GrinddError, Result};

pub fn run_host_command(cmd: &[String]) -> Result<i32> {
    if cmd.is_empty() {
        return Err(GrinddError::InvalidCommand(
            "run requires at least one argument".to_string(),
        ));
    }

    let mut command = Command::new(&cmd[0]);
    if cmd.len() > 1 {
        command.args(&cmd[1..]);
    }

    let status = command.status()?;
    Ok(status.code().unwrap_or(1))
}
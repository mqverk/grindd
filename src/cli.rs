use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "grindd", version, about = "A minimal container engine")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start daemon in foreground mode.
    Daemon,
    /// Run a minimal workload command.
    Run {
        /// Command to run.
        #[arg(required = true)]
        cmd: Vec<String>,
    },
}
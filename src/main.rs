mod cli;
mod daemon;
mod error;
mod runtime;

use clap::Parser;
use cli::{Cli, Commands};
use daemon::{DaemonConfig, ensure_root_layout};
use error::Result;
use runtime::process::run_host_command;

fn main() {
    if let Err(err) = run() {
        eprintln!("grindd: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Daemon => {
            let cfg = DaemonConfig::default();
            ensure_root_layout(&cfg)?;
            println!("daemon state initialized at {}", cfg.root_dir.display());
        }
        Commands::Run { cmd } => {
            let code = run_host_command(&cmd)?;
            std::process::exit(code);
        }
    }

    Ok(())
}

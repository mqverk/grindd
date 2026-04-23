use std::path::PathBuf;

use clap::{Parser, Subcommand};
use grindd_core::config::GrinddConfig;
use grindd_core::engine::{Engine, RunRequest};
use grindd_core::logging::init_logging;

#[derive(Debug, Parser)]
#[command(name = "grindd", version, about = "grindd container engine")]
struct Cli {
    /// Path to a grindd config file in JSON format.
    #[arg(long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run daemon in foreground mode.
    Daemon,
    /// Validate and print effective configuration.
    Config,
    /// Load a tar image into local image store.
    ImageLoad {
        #[arg(long)]
        name: String,
        #[arg(long)]
        tar: PathBuf,
    },
    /// Run a container from an image.
    Run {
        #[arg(long)]
        id: String,
        #[arg(long)]
        image: String,
        #[arg(long)]
        memory: Option<u64>,
        #[arg(long)]
        cpu_quota: Option<u64>,
        #[arg(required = true, trailing_var_arg = true)]
        cmd: Vec<String>,
    },
    /// List containers.
    Ps,
    /// Show container logs.
    Logs {
        #[arg(long)]
        id: String,
    },
    /// Execute command in container context (placeholder).
    Exec {
        #[arg(long)]
        id: String,
        #[arg(required = true, trailing_var_arg = true)]
        cmd: Vec<String>,
    },
    /// Remove container metadata.
    Rm {
        #[arg(long)]
        id: String,
    },
    /// Setup default container network objects.
    NetSetup {
        #[arg(long)]
        id: String,
    },
    /// Build image layers from a Grindfile.
    Build {
        #[arg(long)]
        context: PathBuf,
        #[arg(long)]
        file: PathBuf,
    },
    /// Inspect container metadata.
    Inspect {
        #[arg(long)]
        id: String,
    },
    /// Explain container lifecycle plan.
    Explain {
        #[arg(long)]
        id: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = GrinddConfig::load(cli.config.as_deref())?;
    init_logging(&config.log_level);
    let engine = Engine::new(config.state_root.clone());
    engine.bootstrap()?;

    match cli.command {
        Command::Daemon => {
            println!("daemon mode requested, state root={}", config.state_root.display());
            let _ = engine.daemon.serve_once();
        }
        Command::Config => {
            println!("effective config: {:?}", config);
        }
        Command::ImageLoad { name, tar } => {
            let meta = engine.load_image(&name, &tar)?;
            println!("loaded image={} digest={}", meta.name, meta.digest);
        }
        Command::Run {
            id,
            image,
            memory,
            cpu_quota,
            cmd,
        } => {
            let req = RunRequest {
                id,
                image,
                command: cmd,
                memory_max: memory,
                cpu_quota,
            };
            let code = engine.run_container(&req)?;
            println!("exit code: {code}");
        }
        Command::Ps => {
            let rows = engine.list_containers()?;
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        Command::Logs { id } => {
            let logs = engine.container_logs(&id)?;
            print!("{logs}");
        }
        Command::Exec { id, cmd } => {
            println!("exec requested for container={id} cmd={cmd:?}");
        }
        Command::Rm { id } => {
            engine.remove_container(&id)?;
            println!("removed container={id}");
        }
        Command::NetSetup { id } => {
            engine.setup_default_network(&id)?;
            println!("network configured for container={id}");
        }
        Command::Build { context, file } => {
            let layers = engine.build_from_file(&context, &file)?;
            println!("{}", serde_json::to_string_pretty(&layers)?);
        }
        Command::Inspect { id } => {
            let meta = engine.inspect(&id)?;
            println!("{}", serde_json::to_string_pretty(&meta)?);
        }
        Command::Explain { id } => {
            let report = engine.explain(&id);
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }

    Ok(())
}

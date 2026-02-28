use anyhow::Result;
use clap::{Parser, Subcommand};

mod agent;
mod executor;
mod orchestrator;
mod server;
mod tui;

#[derive(Parser)]
#[command(name = "opencode-parallel")]
#[command(about = "Run multiple AI coding agents in parallel", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the interactive TUI
    Tui {
        /// Number of parallel agents to run
        #[arg(short, long, default_value_t = 4)]
        agents: usize,
        
        /// Working directory
        #[arg(short, long)]
        workdir: Option<String>,
    },
    
    /// Run a batch of tasks from a config file
    Run {
        /// Path to task configuration file (JSON)
        #[arg(short, long)]
        config: String,
        
        /// Maximum parallel agents
        #[arg(short, long, default_value_t = 4)]
        parallel: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Tui { agents, workdir }) => {
            let workdir = workdir.unwrap_or_else(|| ".".to_string());
            tui::run_tui(agents, &workdir).await?;
        }
        Some(Commands::Run { config, parallel }) => {
            executor::run_batch(&config, parallel).await?;
        }
        None => {
            tui::run_tui(4, ".").await?;
        }
    }

    Ok(())
}

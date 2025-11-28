use anyhow::Result;
use clap::{Parser, Subcommand};
use pwnagotchi_core::{Agent, AgentConfig};
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "pwnagotchi")]
#[command(author = "Rust Port")]
#[command(version = "2.8.9")]
#[command(about = "WiFi handshake capture tool for Raspberry Pi", long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Debug mode
    #[arg(short, long)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the pwnagotchi agent
    Start {
        /// Run in manual mode
        #[arg(short, long)]
        manual: bool,
    },
    
    /// Show version information
    Version,
    
    /// Validate configuration file
    CheckConfig {
        /// Configuration file to check
        #[arg(value_name = "FILE")]
        config: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.debug {
        Level::DEBUG
    } else if cli.verbose {
        Level::INFO
    } else {
        Level::WARN
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(true)
        .init();

    match &cli.command {
        Some(Commands::Start { manual }) => {
            info!("Starting pwnagotchi...");
            
            let config = if let Some(config_path) = &cli.config {
                load_config(config_path)?
            } else {
                info!("Using default configuration");
                AgentConfig::default()
            };

            let mut agent = Agent::new(config)?;
            
            if *manual {
                info!("Running in MANUAL mode");
            }

            agent.start().await?;
        }
        
        Some(Commands::Version) => {
            println!("pwnagotchi v2.8.9 (Rust)");
            println!("WiFi handshake capture tool");
        }
        
        Some(Commands::CheckConfig { config }) => {
            info!("Checking configuration: {:?}", config);
            let _cfg = load_config(config)?;
            println!("✓ Configuration is valid");
        }
        
        None => {
            // Default: show help
            println!("Use --help for usage information");
        }
    }

    Ok(())
}

fn load_config(path: &PathBuf) -> Result<AgentConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: AgentConfig = toml::from_str(&content)?;
    Ok(config)
}

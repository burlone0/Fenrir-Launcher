mod commands;

use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "fenrir", about = "Fenrir Game Launcher", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Scan a directory for games
    Scan {
        /// Directory to scan (overrides config)
        #[arg(short, long)]
        path: Option<std::path::PathBuf>,
    },
    /// List all games in library
    List,
    /// Show game details
    Info {
        /// Game title or UUID
        game: String,
    },
    /// Add a game manually
    Add {
        /// Path to game directory
        path: std::path::PathBuf,
    },
    /// Show or modify configuration
    Config {
        /// Key to set
        #[arg(short, long)]
        set: Option<String>,
        /// Value to set
        #[arg(short, long)]
        value: Option<String>,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Scan { path } => commands::scan::run(path),
        Commands::List => commands::list::run(),
        Commands::Info { game } => commands::info::run(&game),
        Commands::Add { path } => commands::add::run(&path),
        Commands::Config { set, value } => commands::config_cmd::run(set, value),
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

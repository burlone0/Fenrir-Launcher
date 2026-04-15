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
    /// Confirm a low-confidence game and add it to the library
    Confirm {
        /// Game title or UUID
        query: String,
    },
    /// Configure a game (create prefix + apply tuning profile)
    Configure {
        /// Game title or UUID
        query: String,
    },
    /// Launch a configured game
    Launch {
        /// Game title or UUID
        query: String,
    },
    /// Manage Wine/Proton runtimes
    Runtime {
        #[command(subcommand)]
        action: RuntimeAction,
    },
}

#[derive(clap::Subcommand)]
enum RuntimeAction {
    /// List available runtimes
    List,
    /// Set default runtime
    SetDefault {
        /// Runtime ID
        id: String,
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
        Commands::Confirm { ref query } => commands::confirm::run(query),
        Commands::Configure { ref query } => commands::configure::run(query),
        Commands::Launch { ref query } => commands::launch::run(query),
        Commands::Runtime { ref action } => match action {
            RuntimeAction::List => commands::runtime::list(),
            RuntimeAction::SetDefault { ref id } => commands::runtime::set_default(id),
        },
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        if let Some(hint) = extract_suggestion(e.as_ref()) {
            eprintln!("hint: {}", hint);
        }
        std::process::exit(1);
    }
}

/// Walks the error source chain looking for a `FenrirError` with a suggestion.
fn extract_suggestion(e: &(dyn std::error::Error + 'static)) -> Option<&'static str> {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(e);
    while let Some(err) = current {
        if let Some(fenrir_err) = err.downcast_ref::<fenrir_core::FenrirError>() {
            return fenrir_err.suggestion();
        }
        current = err.source();
    }
    None
}

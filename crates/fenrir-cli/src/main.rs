use clap::Parser;

#[derive(Parser)]
#[command(name = "fenrir", about = "Fenrir Game Launcher", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {}

fn main() {
    let _cli = Cli::parse();
    println!("Fenrir v{}", env!("CARGO_PKG_VERSION"));
}

use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::library::db::Database;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = FenrirConfig::load()?;
    let db = Database::open(&config.general.library_db)?;
    let games = db.list_games()?;

    if games.is_empty() {
        println!("library is empty. Run 'fenrir scan' to detect games.");
        return Ok(());
    }

    println!(
        "{:<36} {:<30} {:<8} {:<10} CRACK",
        "ID", "TITLE", "STORE", "STATUS"
    );

    for game in &games {
        println!(
            "{:<36} {:<30} {:<8} {:<10} {}",
            game.id,
            truncate(&game.title, 28),
            game.store_origin,
            game.status,
            game.crack_type
                .map(|c| format!("{}", c))
                .unwrap_or_else(|| "-".to_string()),
        );
    }

    println!("\n{} games in library", games.len());
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}

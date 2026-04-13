use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::library::db::Database;

pub fn run(query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = FenrirConfig::load()?;
    let db = Database::open(&config.general.library_db)?;

    let game = if let Ok(uuid) = uuid::Uuid::parse_str(query) {
        db.get_game(uuid)?
    } else {
        db.find_by_title(query)?.into_iter().next()
    };

    match game {
        Some(g) => {
            println!("Title:       {}", g.title);
            println!("ID:          {}", g.id);
            println!("Store:       {}", g.store_origin);
            println!(
                "Crack:       {}",
                g.crack_type
                    .map(|c| format!("{}", c))
                    .unwrap_or_else(|| "-".to_string())
            );
            println!("Status:      {}", g.status);
            println!("Executable:  {}", g.executable.display());
            println!("Install dir: {}", g.install_dir.display());
            let prefix_display = if g.prefix_path.as_os_str().is_empty() {
                "not configured".to_string()
            } else {
                g.prefix_path.to_string_lossy().to_string()
            };
            println!("Prefix:      {}", prefix_display);
            println!("Runtime:     {}", g.runtime_id.as_deref().unwrap_or("auto"));
            println!(
                "Play time:   {}h {}m",
                g.play_time / 3600,
                (g.play_time % 3600) / 60
            );
            if let Some(last) = g.last_played {
                println!("Last played: {}", last.format("%Y-%m-%d %H:%M"));
            }
        }
        None => {
            eprintln!("game not found: {}", query);
        }
    }

    Ok(())
}

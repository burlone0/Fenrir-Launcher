use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::library::db::Database;
use fenrir_core::library::game::{Game, GameStatus, StoreOrigin};
use std::path::Path;

pub fn run(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        eprintln!("path does not exist: {}", path.display());
        return Ok(());
    }

    let config = FenrirConfig::load()?;
    let db = Database::open(&config.general.library_db)?;

    let title = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let exe = find_main_exe(path);

    let game = Game {
        id: uuid::Uuid::new_v4(),
        title: title.clone(),
        executable: exe.unwrap_or_else(|| path.to_path_buf()),
        install_dir: path.to_path_buf(),
        store_origin: StoreOrigin::Unknown,
        crack_type: None,
        prefix_path: std::path::PathBuf::new(),
        runtime_id: None,
        status: GameStatus::Detected,
        play_time: 0,
        last_played: None,
        added_at: chrono::Utc::now(),
        user_overrides: None,
    };

    db.insert_game(&game)?;
    println!("added '{}' ({})", title, game.id);

    Ok(())
}

fn find_main_exe(dir: &Path) -> Option<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("exe"))
                .unwrap_or(false)
        })
}

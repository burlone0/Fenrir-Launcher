use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::library::db::Database;
use fenrir_core::library::game::{Game, GameStatus};
use fenrir_core::scanner;
use fenrir_core::scanner::signatures;
use std::path::PathBuf;

pub fn run(path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let config = FenrirConfig::load()?;

    let scan_dirs = match path {
        Some(p) => vec![p],
        None => {
            if config.scan.game_dirs.is_empty() {
                eprintln!(
                    "no scan directories configured. Use --path or set scan.game_dirs in config."
                );
                return Ok(());
            }
            config.scan.game_dirs.clone()
        }
    };

    let sigs = load_signatures()?;
    println!("loaded {} signatures", sigs.len());

    let db = Database::open(&config.general.library_db)?;

    for dir in &scan_dirs {
        println!("scanning {}...", dir.display());
        let result = scanner::scan_directory(dir, &sigs, 4)?;
        println!(
            "found {} candidates, {} high confidence, {} need confirmation",
            result.total_candidates,
            result.high_confidence.len(),
            result.needs_confirmation.len()
        );

        for classified in &result.high_confidence {
            let game = Game {
                id: uuid::Uuid::new_v4(),
                title: classified.title.clone(),
                executable: classified.exe_files.first().cloned().unwrap_or_default(),
                install_dir: classified.path.clone(),
                store_origin: classified.store_origin,
                crack_type: classified.crack_type,
                prefix_path: PathBuf::new(),
                runtime_id: None,
                status: GameStatus::Detected,
                play_time: 0,
                last_played: None,
                added_at: chrono::Utc::now(),
                user_overrides: None,
            };

            db.insert_game(&game)?;
            println!(
                "  [+] {} ({}, {:?}) — confidence: {}",
                classified.title,
                classified.store_origin,
                classified.crack_type,
                classified.confidence,
            );
        }

        if !result.needs_confirmation.is_empty() {
            println!("\nNeed confirmation:");
            for classified in &result.needs_confirmation {
                println!(
                    "  [?] {} ({:?}) — confidence: {} — {}",
                    classified.title,
                    classified.crack_type,
                    classified.confidence,
                    classified.path.display(),
                );
            }
        }
    }

    Ok(())
}

fn load_signatures() -> Result<Vec<signatures::Signature>, Box<dyn std::error::Error>> {
    // Try relative to binary first, then CWD
    let candidates = [
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("../../../data/signatures"))),
        Some(PathBuf::from("data/signatures")),
    ];

    for candidate in candidates.iter().flatten() {
        if candidate.exists() {
            return Ok(signatures::load_signatures_from_dir(candidate)?);
        }
    }

    Err("signatures directory not found. Ensure data/signatures/ exists.".into())
}

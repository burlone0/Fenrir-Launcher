use crate::AppState;
use fenrir_core::library::game::{Game, GameStatus};
use fenrir_core::scanner::signatures;
use fenrir_core::scanner::{self, ClassifiedGame};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ScanDonePayload {
    pub high_confidence: Vec<ClassifiedGame>,
    pub needs_confirmation: Vec<ClassifiedGame>,
    pub total: usize,
}

#[tauri::command]
pub async fn scan_directory(
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<ScanDonePayload, String> {
    let scan_dirs: Vec<PathBuf> = match path {
        Some(p) => vec![PathBuf::from(p)],
        None => {
            let config = state.config.lock().map_err(|e| e.to_string())?;
            if config.scan.game_dirs.is_empty() {
                return Err("no scan directories configured and no path provided".to_string());
            }
            config.scan.game_dirs.clone()
        }
    };

    for dir in &scan_dirs {
        if !dir.exists() || !dir.is_dir() {
            return Err(format!("directory not found: {}", dir.display()));
        }
    }

    // Load signatures off the async runtime — disk IO + TOML parse.
    let sigs = Arc::new(
        tokio::task::spawn_blocking(load_signatures)
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?,
    );

    let mut all_high: Vec<ClassifiedGame> = Vec::new();
    let mut all_low: Vec<ClassifiedGame> = Vec::new();
    let mut total = 0;

    for dir in scan_dirs {
        let sigs_clone = sigs.clone();
        let result =
            tokio::task::spawn_blocking(move || scanner::scan_directory(&dir, &sigs_clone, 6))
                .await
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?;

        total += result.total_candidates;

        // DB upsert: lock held briefly per directory's results.
        {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            for classified in &result.high_confidence {
                let exe = classified.exe_files.first().cloned().unwrap_or_default();
                match db
                    .find_by_install_dir(&classified.path)
                    .map_err(|e| e.to_string())?
                {
                    Some(mut existing) => {
                        existing.title = classified.title.clone();
                        existing.executable = exe;
                        existing.store_origin = classified.store_origin;
                        existing.crack_type = classified.crack_type;
                        if existing.status == GameStatus::NeedsConfirmation {
                            existing.status = GameStatus::Detected;
                        }
                        db.update_game(&existing).map_err(|e| e.to_string())?;
                    }
                    None => {
                        let game = Game {
                            id: Uuid::new_v4(),
                            title: classified.title.clone(),
                            executable: exe,
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
                        db.insert_game(&game).map_err(|e| e.to_string())?;
                    }
                }
            }
            for classified in &result.needs_confirmation {
                let exe = classified.exe_files.first().cloned().unwrap_or_default();
                if db
                    .find_by_install_dir(&classified.path)
                    .map_err(|e| e.to_string())?
                    .is_none()
                {
                    let game = Game {
                        id: Uuid::new_v4(),
                        title: classified.title.clone(),
                        executable: exe,
                        install_dir: classified.path.clone(),
                        store_origin: classified.store_origin,
                        crack_type: classified.crack_type,
                        prefix_path: PathBuf::new(),
                        runtime_id: None,
                        status: GameStatus::NeedsConfirmation,
                        play_time: 0,
                        last_played: None,
                        added_at: chrono::Utc::now(),
                        user_overrides: None,
                    };
                    db.insert_game(&game).map_err(|e| e.to_string())?;
                }
            }
        }

        all_high.extend(result.high_confidence);
        all_low.extend(result.needs_confirmation);
    }

    Ok(ScanDonePayload {
        high_confidence: all_high,
        needs_confirmation: all_low,
        total,
    })
}

fn load_signatures() -> Result<Vec<signatures::Signature>, Box<dyn std::error::Error + Send + Sync>>
{
    let candidates = [
        std::env::current_exe()
            .ok()
            .and_then(|p| {
                p.parent()
                    .map(|d| d.join("../../data/signatures").canonicalize().ok())
            })
            .flatten(),
        dirs::data_dir().map(|d| d.join("fenrir/signatures")),
        Some(PathBuf::from("data/signatures")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.exists() {
            return Ok(signatures::load_signatures_from_dir(&candidate)?);
        }
    }

    Err("signatures directory not found".into())
}

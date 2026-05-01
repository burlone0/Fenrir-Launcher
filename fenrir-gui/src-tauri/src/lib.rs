mod commands;

use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::library::db::Database;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Database>,
    pub config: Mutex<FenrirConfig>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = FenrirConfig::load().unwrap_or_else(|e| {
        eprintln!("warning: failed to load config ({e}), using defaults");
        FenrirConfig::default()
    });

    if let Some(parent) = config.general.library_db.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!(
                "warning: failed to create library db parent dir {}: {e}",
                parent.display()
            );
        }
    }

    let db = match Database::open(&config.general.library_db) {
        Ok(db) => db,
        Err(e) => {
            eprintln!(
                "fatal: failed to open library database at {}: {e}",
                config.general.library_db.display()
            );
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            db: Mutex::new(db),
            config: Mutex::new(config),
        })
        .invoke_handler(tauri::generate_handler![
            commands::games::list_games,
            commands::games::get_game,
            commands::games::confirm_game,
            commands::games::delete_game,
            commands::games::configure_game,
            commands::games::launch_game,
            commands::scan::scan_directory,
            commands::runtime::list_runtimes,
            commands::runtime::available_runtimes,
            commands::runtime::set_default_runtime,
            commands::runtime::install_runtime,
            commands::config::get_config,
            commands::config::set_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

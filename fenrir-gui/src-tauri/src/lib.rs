mod commands;

use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::library::db::Database;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Database>,
    pub config: FenrirConfig,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = FenrirConfig::load().expect("failed to load fenrir config");
    let db = Database::open(&config.general.library_db).expect("failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            db: Mutex::new(db),
            config,
        })
        .invoke_handler(tauri::generate_handler![
            commands::games::list_games,
            commands::games::get_game,
            commands::games::confirm_game,
            commands::games::delete_game,
            commands::scan::scan_directory,
            commands::runtime::list_runtimes,
            commands::runtime::available_runtimes,
            commands::runtime::set_default_runtime,
            commands::config::get_config,
            commands::config::set_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

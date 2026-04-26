use crate::AppState;
use fenrir_core::library::game::{Game, GameStatus};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_games(state: State<'_, AppState>) -> Result<Vec<Game>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_games().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_game(state: State<'_, AppState>, id: String) -> Result<Game, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_game(uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("game not found: {id}"))
}

#[tauri::command]
pub async fn confirm_game(state: State<'_, AppState>, query: String) -> Result<Game, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut matches = db.find_by_title(&query).map_err(|e| e.to_string())?;
    let game = matches
        .iter_mut()
        .find(|g| g.status == GameStatus::NeedsConfirmation)
        .ok_or_else(|| format!("no unconfirmed game matching: {query}"))?;
    game.status = GameStatus::Detected;
    db.update_game(game).map_err(|e| e.to_string())?;
    Ok(game.clone())
}

#[tauri::command]
pub async fn delete_game(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_game(uuid).map_err(|e| e.to_string())
}

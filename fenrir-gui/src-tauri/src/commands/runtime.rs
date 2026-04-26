use crate::AppState;
use fenrir_core::runtime::discovery;
use fenrir_core::runtime::github;
use fenrir_core::runtime::{GitHubRelease, Runtime};
use tauri::State;

#[tauri::command]
pub async fn list_runtimes(_state: State<'_, AppState>) -> Result<Vec<Runtime>, String> {
    let runtime_dir = dirs::data_dir()
        .map(|d| d.join("fenrir/runtimes"))
        .unwrap_or_default();
    Ok(discovery::discover_all(&runtime_dir))
}

#[tauri::command]
pub async fn available_runtimes(kind: String) -> Result<Vec<GitHubRelease>, String> {
    let client = reqwest::Client::new();
    let releases = match kind.as_str() {
        "proton-ge" => github::list_proton_ge_releases(&client, 15)
            .await
            .map_err(|e| e.to_string())?,
        "wine-ge" => github::list_wine_ge_releases(&client, 15)
            .await
            .map_err(|e| e.to_string())?,
        other => return Err(format!("unknown runtime kind: {other}")),
    };
    Ok(releases)
}

#[tauri::command]
pub async fn set_default_runtime(state: State<'_, AppState>, id: String) -> Result<(), String> {
    // Persist default runtime ID to config — writes back to disk
    let mut config = state.config.clone();
    config.defaults.runtime = id;
    config.save().map_err(|e| e.to_string())
}

use crate::AppState;
use fenrir_core::config::settings::FenrirConfig;
use tauri::State;

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<FenrirConfig, String> {
    Ok(state.config.clone())
}

#[tauri::command]
pub async fn set_config(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let mut config = state.config.clone();
    match key.as_str() {
        "defaults.runtime" => config.defaults.runtime = value,
        "defaults.enable_dxvk" => {
            config.defaults.enable_dxvk = value
                .parse()
                .map_err(|e: std::str::ParseBoolError| e.to_string())?
        }
        "defaults.enable_vkd3d" => {
            config.defaults.enable_vkd3d = value
                .parse()
                .map_err(|e: std::str::ParseBoolError| e.to_string())?
        }
        "defaults.esync" => {
            config.defaults.esync = value
                .parse()
                .map_err(|e: std::str::ParseBoolError| e.to_string())?
        }
        "defaults.fsync" => {
            config.defaults.fsync = value
                .parse()
                .map_err(|e: std::str::ParseBoolError| e.to_string())?
        }
        other => return Err(format!("unknown config key: {other}")),
    }
    config.save().map_err(|e| e.to_string())
}

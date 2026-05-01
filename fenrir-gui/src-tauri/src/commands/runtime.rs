use crate::AppState;
use fenrir_core::runtime::discovery;
use fenrir_core::runtime::downloader::download_runtime;
use fenrir_core::runtime::github;
use fenrir_core::runtime::{GitHubRelease, Runtime};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn list_runtimes(state: State<'_, AppState>) -> Result<Vec<Runtime>, String> {
    let runtime_dir = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.general.runtime_dir.clone()
    };
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

#[derive(Serialize, Clone)]
struct DownloadProgressPayload {
    bytes_received: u64,
    total_bytes: u64,
}

#[derive(Serialize, Clone)]
struct DownloadDonePayload {
    version: String,
}

#[tauri::command]
pub async fn install_runtime(
    app: AppHandle,
    state: State<'_, AppState>,
    version: String,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    let release = find_release(&client, &version).await?;

    let runtime_dir = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.general.runtime_dir.clone()
    };
    tokio::fs::create_dir_all(&runtime_dir)
        .await
        .map_err(|e| e.to_string())?;

    let version_clone = version.clone();

    // Use a channel so the progress callback (Send but not Sync) doesn't cross
    // an await boundary inside the Tauri command future.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(u64, u64)>();
    let app_events = app.clone();
    let events_task = tokio::spawn(async move {
        while let Some((received, total)) = rx.recv().await {
            let _ = app_events.emit(
                "download:progress",
                DownloadProgressPayload {
                    bytes_received: received,
                    total_bytes: total,
                },
            );
        }
    });

    let progress = Box::new({
        let tx = tx.clone();
        move |received: u64, total: u64| {
            let _ = tx.send((received, total));
        }
    });

    let download_result = download_runtime(&client, &release, &runtime_dir, Some(progress)).await;

    // Drop our sender + the original tx so events_task drains and exits even if
    // download_runtime stored its own clone of the callback somewhere.
    drop(tx);
    let _ = events_task.await;

    download_result.map_err(|e| e.to_string())?;

    let _ = app.emit(
        "download:done",
        DownloadDonePayload {
            version: version_clone,
        },
    );

    Ok(())
}

async fn find_release(client: &reqwest::Client, version: &str) -> Result<GitHubRelease, String> {
    let mut errors: Vec<String> = Vec::new();

    match github::list_proton_ge_releases(client, 30).await {
        Ok(releases) => {
            if let Some(r) = releases.into_iter().find(|r| r.tag_name == version) {
                return Ok(r);
            }
        }
        Err(e) => errors.push(format!("proton-ge: {e}")),
    }

    match github::list_wine_ge_releases(client, 30).await {
        Ok(releases) => {
            if let Some(r) = releases.into_iter().find(|r| r.tag_name == version) {
                return Ok(r);
            }
        }
        Err(e) => errors.push(format!("wine-ge: {e}")),
    }

    if errors.is_empty() {
        Err(format!("release not found: {version}"))
    } else {
        Err(format!(
            "release not found: {version} ({})",
            errors.join("; ")
        ))
    }
}

#[tauri::command]
pub async fn set_default_runtime(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.defaults.runtime = id;
    config.save().map_err(|e| e.to_string())
}

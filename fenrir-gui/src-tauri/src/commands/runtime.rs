use crate::AppState;
use fenrir_core::runtime::discovery;
use fenrir_core::runtime::downloader::download_runtime;
use fenrir_core::runtime::github;
use fenrir_core::runtime::{GitHubRelease, Runtime};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

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
pub async fn install_runtime(app: AppHandle, version: String) -> Result<(), String> {
    let client = reqwest::Client::new();

    // Find the release matching version (check both proton-ge and wine-ge)
    let release = find_release(&client, &version).await?;

    let runtime_dir = dirs::data_dir()
        .map(|d| d.join("fenrir/runtimes"))
        .unwrap_or_default();
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

    let progress = Box::new(move |received: u64, total: u64| {
        let _ = tx.send((received, total));
    });

    download_runtime(&client, &release, &runtime_dir, Some(progress))
        .await
        .map_err(|e| e.to_string())?;

    // Drop tx so events_task drains and exits
    events_task.await.ok();

    let _ = app.emit(
        "download:done",
        DownloadDonePayload {
            version: version_clone,
        },
    );

    Ok(())
}

async fn find_release(client: &reqwest::Client, version: &str) -> Result<GitHubRelease, String> {
    let proton = github::list_proton_ge_releases(client, 30).await;
    if let Ok(releases) = proton {
        if let Some(r) = releases.into_iter().find(|r| r.tag_name == version) {
            return Ok(r);
        }
    }
    let wine = github::list_wine_ge_releases(client, 30).await;
    if let Ok(releases) = wine {
        if let Some(r) = releases.into_iter().find(|r| r.tag_name == version) {
            return Ok(r);
        }
    }
    Err(format!("release not found: {version}"))
}

#[tauri::command]
pub async fn set_default_runtime(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut config = state.config.clone();
    config.defaults.runtime = id;
    config.save().map_err(|e| e.to_string())
}

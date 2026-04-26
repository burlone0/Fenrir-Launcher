use crate::AppState;
use fenrir_core::launcher::{launch, monitor_process, LaunchConfig, LaunchResult};
use fenrir_core::library::game::{Game, GameStatus};
use fenrir_core::prefix::{
    apply_profile, crack_type_to_profile_name, create_prefix, load_profiles_from_dir,
    prefix_path_for_game,
};
use fenrir_core::runtime::discovery;
use fenrir_core::runtime::Runtime;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

/// Mirror of CLI logic: match by game.runtime_id, then fall back to first available.
fn resolve_runtime(runtime_dir: &Path, runtime_id: Option<&str>) -> Option<Runtime> {
    let runtimes = discovery::discover_all(runtime_dir);
    if let Some(id) = runtime_id {
        if let Some(rt) = runtimes.iter().find(|r| r.id == id) {
            return Some(rt.clone());
        }
    }
    runtimes.into_iter().next()
}

/// Mirror of CLI find_wine_for_prefix_ops.
fn wine_for_prefix(rt: &Runtime, is_proton: bool) -> PathBuf {
    if is_proton {
        let internal = rt.path.join("files/bin/wine");
        if internal.exists() {
            return internal;
        }
    }
    let wine = rt.path.join("bin/wine");
    if wine.exists() {
        return wine;
    }
    PathBuf::from("/usr/bin/wine")
}

/// Mirror of CLI find_wine_binary (used for launch, not prefix ops).
fn wine_binary(rt: &Runtime) -> PathBuf {
    let proton = rt.path.join("proton");
    if proton.exists() {
        return proton;
    }
    let wine = rt.path.join("bin/wine");
    if wine.exists() {
        return wine;
    }
    PathBuf::from("/usr/bin/wine")
}

// ── Read commands ────────────────────────────────────────────────────────────

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

// ── configure_game ───────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
struct ConfigureStepPayload {
    step: String,
}

#[derive(Serialize, Clone)]
struct ConfigureDonePayload {
    game: Game,
}

#[tauri::command]
pub async fn configure_game(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    clean: bool,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    let (game, runtime, prefix_dir, profiles_dir) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let game = db
            .get_game(uuid)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("game not found: {id}"))?;

        let runtime_dir = &state.config.general.runtime_dir;
        let runtime =
            resolve_runtime(runtime_dir, game.runtime_id.as_deref()).ok_or_else(|| {
                "no Wine/Proton runtime found. Install one from the Runtimes tab.".to_string()
            })?;

        let prefix_dir = state
            .config
            .general
            .library_db
            .parent()
            .unwrap_or(runtime_dir)
            .join("prefixes");
        let profiles_dir = dirs::data_dir()
            .map(|d| d.join("fenrir/profiles"))
            .unwrap_or(PathBuf::from("data/profiles"));

        (game, runtime, prefix_dir, profiles_dir)
    };

    let emit = |step: &str| {
        let _ = app.emit(
            "configure:step",
            ConfigureStepPayload {
                step: step.to_string(),
            },
        );
    };

    emit("creating prefix");
    let prefix_path = prefix_path_for_game(&prefix_dir, game.id);
    let is_proton = matches!(
        runtime.runtime_type,
        fenrir_core::runtime::RuntimeType::Proton | fenrir_core::runtime::RuntimeType::ProtonGE
    );
    let wine_bin = wine_for_prefix(&runtime, is_proton);

    let prefix_path_clone = prefix_path.clone();
    let wine_bin_clone = wine_bin.clone();
    tokio::task::spawn_blocking(move || {
        create_prefix(&prefix_path_clone, &wine_bin_clone, is_proton)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    emit("applying profile");
    let profile_name = crack_type_to_profile_name(game.crack_type);
    if profiles_dir.exists() {
        if let Ok(profiles) = load_profiles_from_dir(&profiles_dir) {
            if let Some(profile) = profiles.get(profile_name) {
                apply_profile(&prefix_path, &wine_bin, profile, None).map_err(|e| e.to_string())?;
            }
        }
    }

    emit("saving");
    let mut updated_game = game.clone();
    updated_game.prefix_path = prefix_path;
    updated_game.runtime_id = Some(runtime.id.clone());
    let _ = clean;
    updated_game.status = GameStatus::Configured;

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.update_game(&updated_game).map_err(|e| e.to_string())?;
    }

    let _ = app.emit(
        "configure:done",
        ConfigureDonePayload { game: updated_game },
    );

    Ok(())
}

// ── launch_game ──────────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
struct LaunchStartedPayload {
    game_id: String,
}

#[derive(Serialize, Clone)]
struct LaunchEndedPayload {
    game_id: String,
    exit_code: i32,
    play_time_secs: u64,
}

#[tauri::command]
pub async fn launch_game(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    let (game, runtime) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let game = db
            .get_game(uuid)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("game not found: {id}"))?;

        let runtime = resolve_runtime(
            &state.config.general.runtime_dir,
            game.runtime_id.as_deref(),
        )
        .ok_or_else(|| "no runtime available. Install one from the Runtimes tab.".to_string())?;

        (game, runtime)
    };

    let _ = app.emit(
        "launch:started",
        LaunchStartedPayload {
            game_id: id.clone(),
        },
    );

    let is_proton = matches!(
        runtime.runtime_type,
        fenrir_core::runtime::RuntimeType::Proton | fenrir_core::runtime::RuntimeType::ProtonGE
    );
    let wine_bin = wine_binary(&runtime);
    let proton_path = if is_proton {
        Some(runtime.path.clone())
    } else {
        None
    };

    let steam_app_id = fenrir_core::launcher::read_steam_app_id(&game.install_dir);

    let env_vars = fenrir_core::prefix::build_wine_env(&game.prefix_path, false, false);

    let config = LaunchConfig {
        executable: game.install_dir.join(&game.executable),
        wine_binary: wine_bin,
        prefix_path: game.prefix_path.clone(),
        env_vars,
        is_proton,
        proton_path,
        steam_app_id,
    };

    let log_dir = dirs::data_dir()
        .map(|d| d.join("fenrir/logs"))
        .unwrap_or_default();
    std::fs::create_dir_all(&log_dir).ok();
    let log_path = log_dir.join(format!("{}.log", game.id));

    let child = launch(&config).map_err(|e| e.to_string())?;

    let game_id_clone = id.clone();
    let app_clone = app.clone();

    let result: LaunchResult =
        tokio::task::spawn_blocking(move || monitor_process(child, &log_path))
            .await
            .map_err(|e| e.to_string())?;

    // Update play_time in DB
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        if let Ok(Some(mut g)) = db.get_game(uuid) {
            g.play_time += result.play_time_secs;
            g.last_played = Some(chrono::Utc::now());
            db.update_game(&g).ok();
        }
    }

    let _ = app_clone.emit(
        "launch:ended",
        LaunchEndedPayload {
            game_id: game_id_clone,
            exit_code: result.exit_code.unwrap_or(-1),
            play_time_secs: result.play_time_secs,
        },
    );

    Ok(())
}

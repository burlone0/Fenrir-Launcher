use chrono::Utc;
use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::launcher::monitor;
use fenrir_core::launcher::runner::read_steam_app_id;
use fenrir_core::launcher::{self, LaunchConfig};
use fenrir_core::library::db::Database;
use fenrir_core::library::game::GameStatus;
use fenrir_core::prefix::builder;
use fenrir_core::prefix::profile::load_profiles_from_dir;
use fenrir_core::runtime;
use std::path::PathBuf;

pub fn run(query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = FenrirConfig::load()?;
    let db = Database::open(&config.general.library_db)?;

    let mut game = if let Ok(uuid) = uuid::Uuid::parse_str(query) {
        db.get_game(uuid)?.ok_or("game not found")?
    } else {
        db.find_by_title(query)?
            .into_iter()
            .next()
            .ok_or("game not found")?
    };

    if game.status == GameStatus::Detected {
        eprintln!(
            "game not configured yet. Run 'fenrir configure \"{}\"' first.",
            game.title
        );
        return Ok(());
    }

    // Find the assigned runtime, or fallback to first available
    let runtimes = runtime::discover_all(&config.general.runtime_dir);
    let rt = game
        .runtime_id
        .as_ref()
        .and_then(|id| runtimes.iter().find(|r| &r.id == id))
        .or_else(|| runtimes.first())
        .ok_or("no runtime available")?;

    let is_proton = matches!(
        rt.runtime_type,
        fenrir_core::runtime::RuntimeType::Proton | fenrir_core::runtime::RuntimeType::ProtonGE
    );

    let wine_bin = if is_proton {
        rt.path.join("proton")
    } else {
        rt.path.join("bin/wine")
    };

    // 1. Base env (esync/fsync)
    let mut env = builder::build_wine_env(
        &game.prefix_path,
        config.defaults.esync,
        config.defaults.fsync,
    );

    // 2. Profile env vars
    if let Some(dir) = find_profiles_dir() {
        let profile_name = fenrir_core::prefix::crack_type_to_profile_name(game.crack_type);
        if let Ok(profiles) = load_profiles_from_dir(&dir) {
            if let Some(profile) = profiles.get(profile_name) {
                for (k, v) in &profile.env {
                    env.insert(k.clone(), v.clone());
                }
            }
        }
    }

    // 3. User overrides win over profile
    if let Some(ref overrides) = game.user_overrides {
        if let Some(user_env) = overrides.get("env_vars").and_then(|v| v.as_object()) {
            for (k, v) in user_env {
                if let Some(s) = v.as_str() {
                    env.insert(k.clone(), s.to_string());
                }
            }
        }
    }

    // 4. Steam AppID
    let steam_app_id = read_steam_app_id(&game.install_dir);

    let launch_config = LaunchConfig {
        executable: game.executable.clone(),
        wine_binary: wine_bin,
        prefix_path: game.prefix_path.clone(),
        env_vars: env,
        is_proton,
        proton_path: if is_proton {
            Some(rt.path.clone())
        } else {
            None
        },
        steam_app_id,
    };

    println!("launching '{}'...", game.title);

    let child = launcher::launch(&launch_config)?;

    // Setup log path
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("fenrir/logs");
    std::fs::create_dir_all(&log_dir)?;
    let log_path = log_dir.join(format!("{}.log", game.id));

    let result = monitor::monitor_process(child, &log_path);

    // Update playtime and status
    game.play_time += result.play_time_secs;
    game.last_played = Some(Utc::now());
    if result.exit_code == Some(0) {
        game.status = GameStatus::Ready;
    }
    db.update_game(&game)?;

    println!(
        "game exited (code: {:?}, played: {}m)",
        result.exit_code,
        result.play_time_secs / 60
    );

    Ok(())
}

fn find_profiles_dir() -> Option<PathBuf> {
    let candidates = [
        std::env::current_exe()
            .ok()
            .and_then(|p| {
                p.parent()
                    .map(|p| p.join("../../data/profiles").canonicalize().ok())
            })
            .flatten(),
        Some(PathBuf::from("data/profiles")),
        dirs::data_dir().map(|d| d.join("fenrir/profiles")),
    ];
    candidates.into_iter().flatten().find(|p| p.exists())
}

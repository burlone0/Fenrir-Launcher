use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::library::db::Database;
use fenrir_core::library::game::GameStatus;
use fenrir_core::prefix;
use fenrir_core::prefix::profile::load_profiles_from_dir;
use fenrir_core::runtime::{self, RuntimeType};
use std::path::PathBuf;

pub fn run(query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = FenrirConfig::load()?;
    let db = Database::open(&config.general.library_db)?;

    // Find game by UUID or fuzzy title search
    let mut game = if let Ok(uuid) = uuid::Uuid::parse_str(query) {
        db.get_game(uuid)?.ok_or("game not found")?
    } else {
        db.find_by_title(query)?
            .into_iter()
            .next()
            .ok_or("game not found")?
    };

    if matches!(game.status, GameStatus::Configured | GameStatus::Ready) {
        println!(
            "'{}' is already configured. Re-run with --force to override (not yet implemented).",
            game.title
        );
        return Ok(());
    }

    println!("configuring '{}'...", game.title);

    // 1. Find runtime
    let runtimes = runtime::discover_all(&config.general.runtime_dir);
    let rt = runtimes
        .first()
        .ok_or("no Wine/Proton runtime found. Install one or check 'fenrir runtime list'")?;
    println!("  runtime: {} ({})", rt.id, rt.runtime_type);

    // 2. Create prefix
    let prefix_path = prefix::prefix_path_for_game(&config.general.prefix_dir, game.id);
    // For prefix operations (wineboot, regedit) always use the underlying Wine binary.
    // Proton's wrapper script requires Steam env vars we don't have at configure time.
    let is_proton = matches!(rt.runtime_type, RuntimeType::Proton | RuntimeType::ProtonGE);
    let wine_for_ops = find_wine_for_prefix_ops(rt, is_proton);

    println!("  creating prefix at {}...", prefix_path.display());
    prefix::create_prefix(&prefix_path, &wine_for_ops, false)?;

    // 3. Load and apply profile
    let profiles_dir = find_profiles_dir();
    let profile_name = crack_type_to_profile_name(game.crack_type);

    if let Some(dir) = profiles_dir {
        let profiles = load_profiles_from_dir(&dir)?;
        if let Some(profile) = profiles.get(profile_name) {
            println!("  applying profile '{}'...", profile_name);
            prefix::apply_profile(
                &prefix_path,
                &wine_for_ops,
                profile,
                game.user_overrides.as_ref(),
            )?;
        } else {
            println!("  no profile for '{}', using defaults", profile_name);
        }
    }

    // 4. Update game in DB
    game.prefix_path = prefix_path;
    game.runtime_id = Some(rt.id.clone());
    game.status = GameStatus::Configured;
    db.update_game(&game)?;

    println!("  done! Run 'fenrir launch \"{}\"' to play.", game.title);

    Ok(())
}

/// Returns the launch binary for the runtime (proton script for Proton, wine for Wine).
fn find_wine_binary(rt: &fenrir_core::runtime::Runtime) -> PathBuf {
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

/// Returns the Wine binary suitable for prefix operations (wineboot, regedit).
/// For Proton runtimes, this is the internal files/bin/wine, which works without
/// Steam environment variables that the proton wrapper script requires.
fn find_wine_for_prefix_ops(rt: &fenrir_core::runtime::Runtime, is_proton: bool) -> PathBuf {
    if is_proton {
        let internal = rt.path.join("files/bin/wine");
        if internal.exists() {
            return internal;
        }
    }
    find_wine_binary(rt)
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

fn crack_type_to_profile_name(
    crack_type: Option<fenrir_core::library::game::CrackType>,
) -> &'static str {
    use fenrir_core::library::game::CrackType;
    match crack_type {
        Some(CrackType::OnlineFix) => "onlinefix",
        Some(CrackType::DODI) => "dodi",
        Some(CrackType::FitGirl) => "fitgirl",
        Some(CrackType::Scene) => "scene",
        Some(CrackType::GOGRip) => "gog",
        _ => "steam_generic",
    }
}

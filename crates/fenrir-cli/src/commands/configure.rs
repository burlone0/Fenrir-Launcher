use fenrir_core::cleanup;
use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::library::db::Database;
use fenrir_core::library::game::GameStatus;
use fenrir_core::prefix;
use fenrir_core::prefix::profile::load_profiles_from_dir;
use fenrir_core::runtime::{self, RuntimeType};
use fenrir_core::scanner::classifier::classify_candidate;
use fenrir_core::scanner::detector::GameCandidate;
use fenrir_core::scanner::signatures::load_signatures_from_dir;
use serde_json;
use std::io::{self, Write as IoWrite};
use std::path::PathBuf;

pub fn run(query: &str, clean: bool, yes: bool) -> Result<(), Box<dyn std::error::Error>> {
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

    let already_configured = matches!(game.status, GameStatus::Configured | GameStatus::Ready);

    if already_configured && !clean {
        println!(
            "'{}' is already configured. Re-run with --force to override (not yet implemented).",
            game.title
        );
        return Ok(());
    }

    if !already_configured {
        println!("configuring '{}'...", game.title);

        // 1. Find runtime
        let runtimes = runtime::discover_all(&config.general.runtime_dir);
        let rt = runtimes
            .first()
            .ok_or("no Wine/Proton runtime found. Install one or check 'fenrir runtime list'")?;
        println!("  runtime: {} ({})", rt.id, rt.runtime_type);

        // 2. Create prefix
        let prefix_path = prefix::prefix_path_for_game(&config.general.prefix_dir, game.id);
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
    }

    if clean {
        run_cleanup(&mut game, &db, yes)?;
    }

    Ok(())
}

fn run_cleanup(
    game: &mut fenrir_core::library::game::Game,
    db: &Database,
    yes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Re-classify the install dir to get the winning signature's cleanup_files
    let sig_dir = match find_signatures_dir() {
        Some(d) => d,
        None => {
            eprintln!("warning: signatures directory not found, skipping cleanup");
            return Ok(());
        }
    };

    let signatures = load_signatures_from_dir(&sig_dir)?;
    let candidate = GameCandidate {
        path: game.install_dir.clone(),
        exe_files: vec![],
    };

    let cleanup_files = classify_candidate(&candidate, &signatures)
        .map(|(_, classified)| {
            signatures
                .iter()
                .find(|s| s.name == classified.signature_name)
                .map(|s| s.cleanup_files.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default();

    if cleanup_files.is_empty() {
        println!("no cleanup patterns defined for '{}'", game.title);
        return Ok(());
    }

    let plan = cleanup::build_cleanup_plan(&game.install_dir, &cleanup_files);

    if plan.is_empty() {
        println!("nothing to clean in '{}'", game.title);
        return Ok(());
    }

    println!("cleanup preview for '{}':", game.title);
    for entry in &plan.entries {
        if entry.is_dir {
            println!("  remove dir:  {}", entry.path.display());
        } else {
            println!("  remove file: {}", entry.path.display());
        }
    }

    let size = plan.total_size_bytes();
    println!(
        "total: {} file(s), {} dir(s) (~{} MB)",
        plan.file_count(),
        plan.dir_count(),
        size / 1_048_576,
    );

    if !yes {
        print!("\nproceed? [y/N] ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("cleanup aborted.");
            return Ok(());
        }
    }

    let result = cleanup::execute_cleanup(&plan);
    println!(
        "cleanup done: {} removed, {} errors",
        result.removed, result.errors
    );

    // Mark cleanup done in user_overrides
    let mut overrides = game
        .user_overrides
        .take()
        .unwrap_or_else(|| serde_json::json!({}));
    overrides["cleanup_done"] = serde_json::json!(true);
    game.user_overrides = Some(overrides);
    db.update_game(game)?;

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

fn find_signatures_dir() -> Option<PathBuf> {
    let candidates = [
        std::env::current_exe()
            .ok()
            .and_then(|p| {
                p.parent()
                    .map(|p| p.join("../../data/signatures").canonicalize().ok())
            })
            .flatten(),
        Some(PathBuf::from("data/signatures")),
        dirs::data_dir().map(|d| d.join("fenrir/signatures")),
    ];
    candidates.into_iter().flatten().find(|p| p.exists())
}

fn crack_type_to_profile_name(
    crack_type: Option<fenrir_core::library::game::CrackType>,
) -> &'static str {
    fenrir_core::prefix::crack_type_to_profile_name(crack_type)
}

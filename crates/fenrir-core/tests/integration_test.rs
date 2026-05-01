use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::library::db::Database;
use fenrir_core::library::game::{Game, GameStatus, StoreOrigin};
use fenrir_core::prefix::builder;
use fenrir_core::prefix::profile::{load_profiles_from_dir, WineProfile};
use fenrir_core::runtime::discovery::discover_runtimes_in_dir;
use fenrir_core::runtime::{RuntimeSource, RuntimeType};
use fenrir_core::scanner;
use fenrir_core::scanner::signatures;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

// ---------------------------------------------------------------------------
// Test 1: Full scan pipeline (scan → classify → persist in DB)
// ---------------------------------------------------------------------------

#[test]
fn test_full_scan_pipeline() {
    // Setup: create a fake game directory structure
    let games_dir = tempdir().unwrap();
    let game_dir = games_dir.path().join("Fake Game");
    fs::create_dir(&game_dir).unwrap();
    fs::write(game_dir.join("game.exe"), "fake").unwrap();
    fs::write(game_dir.join("steam_api.dll"), "fake").unwrap();
    fs::write(game_dir.join("steam_api64.dll"), "fake").unwrap();
    fs::write(game_dir.join("steam_appid.txt"), "12345").unwrap();

    // Load signatures from inline TOML
    let sig_toml = r#"
[steam_generic]
name = "Steam Generic Crack"
store = "Steam"
required_files = ["steam_api.dll"]
optional_files = ["steam_api64.dll", "steam_appid.txt"]
confidence_boost = ["steam_emu.ini"]
"#;
    let sigs = signatures::parse_signatures_from_str(sig_toml).unwrap();

    // Scan
    let result = scanner::scan_directory(games_dir.path(), &sigs, 4).unwrap();

    assert_eq!(result.high_confidence.len(), 1);
    assert_eq!(result.high_confidence[0].title, "Fake Game");
    assert_eq!(result.high_confidence[0].store_origin, StoreOrigin::Steam);
    assert!(result.high_confidence[0].confidence >= 60);

    // Persist to DB
    let db = Database::open_in_memory().unwrap();
    let game = Game {
        id: uuid::Uuid::new_v4(),
        title: result.high_confidence[0].title.clone(),
        executable: result.high_confidence[0].exe_files[0].clone(),
        install_dir: result.high_confidence[0].path.clone(),
        store_origin: result.high_confidence[0].store_origin,
        crack_type: result.high_confidence[0].crack_type,
        prefix_path: PathBuf::new(),
        runtime_id: None,
        status: GameStatus::Detected,
        play_time: 0,
        last_played: None,
        added_at: chrono::Utc::now(),
        user_overrides: None,
    };
    db.insert_game(&game).unwrap();

    // Verify
    let games = db.list_games().unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].title, "Fake Game");
    assert_eq!(games[0].status, GameStatus::Detected);
}

// ---------------------------------------------------------------------------
// Test 2: Config persistence roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_config_persistence() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    let mut config = FenrirConfig::default();
    config.scan.game_dirs = vec![PathBuf::from("/mnt/games")];
    config.privacy.fetch_metadata = true;
    config.save_to(&config_path).unwrap();

    let loaded = FenrirConfig::load_from(&config_path).unwrap();
    assert_eq!(loaded.scan.game_dirs.len(), 1);
    assert_eq!(loaded.scan.game_dirs[0], PathBuf::from("/mnt/games"));
    assert!(loaded.privacy.fetch_metadata);
}

// ---------------------------------------------------------------------------
// Test 3: Runtime discovery pipeline
// ---------------------------------------------------------------------------

#[test]
fn test_runtime_discovery_pipeline() {
    let dir = tempdir().unwrap();

    // Simulate runtime directory structure
    fs::create_dir(dir.path().join("GE-Proton9-20")).unwrap();
    fs::create_dir(dir.path().join("wine-ge-8-26")).unwrap();
    fs::create_dir(dir.path().join("Proton 9.0")).unwrap();
    fs::create_dir(dir.path().join("not-a-runtime")).unwrap();

    let runtimes = discover_runtimes_in_dir(dir.path(), RuntimeSource::Downloaded);

    assert_eq!(runtimes.len(), 3);

    let proton_ge = runtimes.iter().find(|r| r.id == "GE-Proton9-20").unwrap();
    assert_eq!(proton_ge.runtime_type, RuntimeType::ProtonGE);
    assert_eq!(proton_ge.version, "9-20");

    let wine_ge = runtimes.iter().find(|r| r.id == "wine-ge-8-26").unwrap();
    assert_eq!(wine_ge.runtime_type, RuntimeType::WineGE);

    let proton = runtimes.iter().find(|r| r.id == "Proton 9.0").unwrap();
    assert_eq!(proton.runtime_type, RuntimeType::Proton);
}

// ---------------------------------------------------------------------------
// Test 4: Profile loading from disk
// ---------------------------------------------------------------------------

#[test]
fn test_profile_loading_pipeline() {
    let dir = tempdir().unwrap();

    fs::write(
        dir.path().join("steam_generic.toml"),
        r#"
[profile]
name = "steam_generic"
description = "Default profile for Steam cracked games"

[wine]
windows_version = "win10"
dll_overrides = ["steam_api=n", "steam_api64=n"]

[env]

[features]
dxvk = true
vkd3d = false
esync = true
fsync = true
"#,
    )
    .unwrap();

    fs::write(
        dir.path().join("onlinefix.toml"),
        r#"
[profile]
name = "onlinefix"
description = "Profile for OnlineFix cracked games"

[wine]
windows_version = "win10"
dll_overrides = ["steam_api=n", "steam_api64=n", "steamclient=n", "steamclient64=n"]

[env]
OPENSSL_ia32cap = "~0x20000000"

[features]
dxvk = true
vkd3d = false
esync = true
fsync = true
"#,
    )
    .unwrap();

    let profiles = load_profiles_from_dir(dir.path()).unwrap();
    assert_eq!(profiles.len(), 2);

    let steam = &profiles["steam_generic"];
    assert_eq!(steam.wine.dll_overrides.len(), 2);
    assert!(steam.features.dxvk);

    let onlinefix = &profiles["onlinefix"];
    assert_eq!(onlinefix.wine.dll_overrides.len(), 4);
    assert_eq!(onlinefix.env.get("OPENSSL_ia32cap").unwrap(), "~0x20000000");
}

// ---------------------------------------------------------------------------
// Test 5: Prefix env building with profile features
// ---------------------------------------------------------------------------

#[test]
fn test_prefix_env_with_profile() {
    let prefix = PathBuf::from("/tmp/test-game-prefix");

    let profile_toml = r#"
[profile]
name = "test_profile"
description = "Test"

[wine]
windows_version = "win10"
dll_overrides = ["steam_api=n"]

[env]
CUSTOM_VAR = "custom_value"

[features]
dxvk = true
vkd3d = false
esync = true
fsync = true
"#;

    let profile = WineProfile::parse(profile_toml).unwrap();

    // Build base env from profile features
    let env = builder::build_wine_env(&prefix, profile.features.esync, profile.features.fsync);
    assert_eq!(env.get("WINEESYNC").unwrap(), "1");
    assert_eq!(env.get("WINEFSYNC").unwrap(), "1");
    assert_eq!(env.get("WINEPREFIX").unwrap(), "/tmp/test-game-prefix");
}

// ---------------------------------------------------------------------------
// Test 6: Launcher command composition (Wine vs Proton)
// ---------------------------------------------------------------------------

#[test]
fn test_launch_command_wine_vs_proton() {
    use fenrir_core::launcher::runner::{build_launch_command, LaunchConfig};
    use std::collections::HashMap;

    // Wine command
    let wine_cmd = build_launch_command(&LaunchConfig {
        executable: PathBuf::from("/games/elden-ring/game.exe"),
        wine_binary: PathBuf::from("/usr/bin/wine"),
        prefix_path: PathBuf::from("/data/prefixes/uuid-1234"),
        env_vars: HashMap::new(),
        is_proton: false,
        proton_path: None,
        steam_app_id: None,
    });

    assert_eq!(wine_cmd.program, "/usr/bin/wine");
    assert_eq!(wine_cmd.args, vec!["/games/elden-ring/game.exe"]);
    assert_eq!(
        wine_cmd.env.get("WINEPREFIX").unwrap(),
        "/data/prefixes/uuid-1234"
    );
    assert_eq!(wine_cmd.working_dir, PathBuf::from("/games/elden-ring"));

    // Proton command
    let proton_cmd = build_launch_command(&LaunchConfig {
        executable: PathBuf::from("/games/elden-ring/game.exe"),
        wine_binary: PathBuf::from("/runtimes/GE-Proton9-20/proton"),
        prefix_path: PathBuf::from("/data/prefixes/uuid-1234"),
        env_vars: HashMap::new(),
        is_proton: true,
        proton_path: Some(PathBuf::from("/runtimes/GE-Proton9-20")),
        steam_app_id: None,
    });

    assert_eq!(proton_cmd.program, "/runtimes/GE-Proton9-20/proton");
    assert_eq!(proton_cmd.args, vec!["run", "/games/elden-ring/game.exe"]);
    assert!(proton_cmd.env.contains_key("STEAM_COMPAT_DATA_PATH"));
    // Proton should NOT have WINEPREFIX — it uses STEAM_COMPAT_DATA_PATH
    assert!(!proton_cmd.env.contains_key("WINEPREFIX"));
}

// ---------------------------------------------------------------------------
// Test 7: Full flow — scan → DB → configure (prefix + profile) → verify state
// ---------------------------------------------------------------------------

#[test]
fn test_scan_to_configure_flow() {
    // 1. Scan: create fake game
    let games_dir = tempdir().unwrap();
    let game_dir = games_dir.path().join("Cyberpunk 2077");
    fs::create_dir(&game_dir).unwrap();
    fs::write(game_dir.join("Cyberpunk2077.exe"), "fake").unwrap();
    fs::write(game_dir.join("steam_api.dll"), "fake").unwrap();
    fs::write(game_dir.join("steam_api64.dll"), "fake").unwrap();
    fs::write(game_dir.join("steam_appid.txt"), "1091500").unwrap();

    let sigs = signatures::parse_signatures_from_str(
        r#"
[steam_generic]
name = "Steam Generic Crack"
store = "Steam"
required_files = ["steam_api.dll"]
optional_files = ["steam_api64.dll", "steam_appid.txt"]
confidence_boost = []
"#,
    )
    .unwrap();

    let result = scanner::scan_directory(games_dir.path(), &sigs, 4).unwrap();
    assert_eq!(result.high_confidence.len(), 1);

    let classified = &result.high_confidence[0];

    // 2. Insert in DB as Detected
    let db = Database::open_in_memory().unwrap();
    let game_id = uuid::Uuid::new_v4();
    let mut game = Game {
        id: game_id,
        title: classified.title.clone(),
        executable: classified.exe_files[0].clone(),
        install_dir: classified.path.clone(),
        store_origin: classified.store_origin,
        crack_type: classified.crack_type,
        prefix_path: PathBuf::new(),
        runtime_id: None,
        status: GameStatus::Detected,
        play_time: 0,
        last_played: None,
        added_at: chrono::Utc::now(),
        user_overrides: None,
    };
    db.insert_game(&game).unwrap();

    // 3. Simulate configure: assign prefix and runtime
    let prefix_dir = tempdir().unwrap();
    let prefix_path = fenrir_core::prefix::prefix_path_for_game(prefix_dir.path(), game_id);

    game.prefix_path = prefix_path;
    game.runtime_id = Some("GE-Proton9-20".to_string());
    game.status = GameStatus::Configured;
    db.update_game(&game).unwrap();

    // 4. Verify final state
    let fetched = db.get_game(game_id).unwrap().unwrap();
    assert_eq!(fetched.title, "Cyberpunk 2077");
    assert_eq!(fetched.status, GameStatus::Configured);
    assert_eq!(fetched.runtime_id.as_deref(), Some("GE-Proton9-20"));
    assert!(fetched
        .prefix_path
        .to_string_lossy()
        .contains(&game_id.to_string()));
}

// ---------------------------------------------------------------------------
// Test 8: OnlineFix detection → correct profile selection
// ---------------------------------------------------------------------------

#[test]
fn test_onlinefix_detection_and_profile_match() {
    // Scan with OnlineFix signature
    let games_dir = tempdir().unwrap();
    let game_dir = games_dir.path().join("Some Game");
    fs::create_dir(&game_dir).unwrap();
    fs::write(game_dir.join("game.exe"), "fake").unwrap();
    fs::write(game_dir.join("OnlineFix.ini"), "fake").unwrap();
    fs::write(game_dir.join("OnlineFix64.dll"), "fake").unwrap();
    fs::write(game_dir.join("steam_api.dll"), "fake").unwrap();

    let sig_toml = r#"
[onlinefix]
name = "OnlineFix"
store = "Steam"
crack_type = "OnlineFix"
required_files = ["OnlineFix.ini"]
optional_files = ["OnlineFix64.dll", "OnlineFix.url", "steamclient.dll"]
confidence_boost = ["steam_settings/"]

[steam_generic]
name = "Steam Generic Crack"
store = "Steam"
required_files = ["steam_api.dll"]
optional_files = ["steam_api64.dll"]
confidence_boost = []
"#;
    let sigs = signatures::parse_signatures_from_str(sig_toml).unwrap();
    let result = scanner::scan_directory(games_dir.path(), &sigs, 4).unwrap();

    // OnlineFix should win (30 required + 15 optional = 45, vs Steam 30)
    let all_games: Vec<_> = result
        .high_confidence
        .iter()
        .chain(result.needs_confirmation.iter())
        .collect();
    assert_eq!(all_games.len(), 1);

    let game = all_games[0];
    assert_eq!(
        game.crack_type,
        Some(fenrir_core::library::game::CrackType::OnlineFix)
    );

    // Verify correct profile would be selected
    let profiles_dir = tempdir().unwrap();
    fs::write(
        profiles_dir.path().join("onlinefix.toml"),
        r#"
[profile]
name = "onlinefix"
description = "OnlineFix profile"
[wine]
windows_version = "win10"
dll_overrides = ["steam_api=n", "steamclient=n"]
[env]
OPENSSL_ia32cap = "~0x20000000"
[features]
dxvk = true
vkd3d = false
esync = true
fsync = true
"#,
    )
    .unwrap();

    let profiles = load_profiles_from_dir(profiles_dir.path()).unwrap();
    assert!(profiles.contains_key("onlinefix"));
    let profile = &profiles["onlinefix"];
    assert!(profile.env.contains_key("OPENSSL_ia32cap"));
}

// ---------------------------------------------------------------------------
// Test 9: steam_generic_64 signature exists in data files and detects 64-bit games
// ---------------------------------------------------------------------------

#[test]
fn test_steam_generic_64_signature_exists_in_data_dir() {
    // CARGO_MANIFEST_DIR = crates/fenrir-core/ → ../../data/signatures = repo root
    let sigs_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/signatures");
    let sigs = signatures::load_signatures_from_dir(&sigs_dir).unwrap();
    let sig = sigs
        .iter()
        .find(|s| s.name == "Steam Generic Crack (64-bit)");
    assert!(
        sig.is_some(),
        "steam_generic_64 signature must exist in data/signatures/ — \
         many modern games ship only steam_api64.dll without steam_api.dll"
    );
    let sig = sig.unwrap();
    assert!(
        sig.required_files.contains(&"steam_api64.dll".to_string()),
        "steam_api64.dll must be a required file"
    );
    assert!(
        !sig.required_files.contains(&"steam_api.dll".to_string()),
        "steam_api.dll must NOT be required in the 64-bit signature"
    );
}

#[test]
fn test_steam_api64_only_game_is_detected() {
    let games_dir = tempdir().unwrap();
    let game_dir = games_dir.path().join("Animal Well");
    fs::create_dir(&game_dir).unwrap();
    fs::write(game_dir.join("AnimalWell.exe"), "fake").unwrap();
    fs::write(game_dir.join("steam_api64.dll"), "fake").unwrap();
    fs::write(game_dir.join("steam_appid.txt"), "813230").unwrap();

    let sig_toml = r#"
[steam_generic_64]
name = "Steam Generic Crack (64-bit)"
store = "Steam"
required_files = ["steam_api64.dll"]
optional_files = ["steam_api.dll", "steam_appid.txt"]
confidence_boost = ["steam_emu.ini", "cream_api.ini"]
"#;
    let sigs = signatures::parse_signatures_from_str(sig_toml).unwrap();
    let result = scanner::scan_directory(games_dir.path(), &sigs, 4).unwrap();

    let all: Vec<_> = result
        .high_confidence
        .iter()
        .chain(result.needs_confirmation.iter())
        .collect();
    assert_eq!(all.len(), 1, "64-bit only game must be detected");
    assert_eq!(
        all[0].store_origin,
        fenrir_core::library::game::StoreOrigin::Steam
    );
    // steam_api64.dll (30) + steam_appid.txt (15) = 45 → needs_confirmation
    assert_eq!(all[0].confidence, 45);
}

// ---------------------------------------------------------------------------
// Test 10: OnlineFix detected without OnlineFix.url (users routinely delete it)
// ---------------------------------------------------------------------------
//
// (Tests 11-14 below cover Fase 2: GOG detection, checksum, GitHub API, profiles)
// ---------------------------------------------------------------------------

#[test]
fn test_onlinefix_detected_without_url_file() {
    let games_dir = tempdir().unwrap();
    let game_dir = games_dir.path().join("Scam Line");
    fs::create_dir(&game_dir).unwrap();
    fs::write(game_dir.join("Scam Line.exe"), "fake").unwrap();
    // OnlineFix.url is intentionally absent — user deleted it
    fs::write(game_dir.join("OnlineFix.ini"), "fake").unwrap();
    fs::write(game_dir.join("OnlineFix64.dll"), "fake").unwrap();

    let sig_toml = r#"
[onlinefix]
name = "OnlineFix"
store = "Steam"
crack_type = "OnlineFix"
required_files = ["OnlineFix.ini"]
optional_files = ["OnlineFix64.dll", "OnlineFix.url", "steamclient.dll"]
confidence_boost = ["steam_settings/"]
"#;
    let sigs = signatures::parse_signatures_from_str(sig_toml).unwrap();
    let result = scanner::scan_directory(games_dir.path(), &sigs, 4).unwrap();

    let all: Vec<_> = result
        .high_confidence
        .iter()
        .chain(result.needs_confirmation.iter())
        .collect();
    assert_eq!(
        all.len(),
        1,
        "game must be detected even without OnlineFix.url"
    );
    assert_eq!(
        all[0].crack_type,
        Some(fenrir_core::library::game::CrackType::OnlineFix)
    );
    // OnlineFix.ini (30) + OnlineFix64.dll (15) = 45 → needs_confirmation
    assert_eq!(all[0].confidence, 45);
}

// ---------------------------------------------------------------------------
// Test 11 (Fase 2): GOG game detection via goggame-*.info glob pattern
// ---------------------------------------------------------------------------

#[test]
fn test_gog_game_detection() {
    let games_dir = tempdir().unwrap();
    let game_dir = games_dir.path().join("The Witcher 3");
    fs::create_dir(&game_dir).unwrap();
    fs::write(game_dir.join("witcher3.exe"), "fake").unwrap();
    // GOG-specific metadata file with a glob-matched name
    fs::write(
        game_dir.join("goggame-1207664643.info"),
        r#"{"gameId":"1207664643"}"#,
    )
    .unwrap();
    fs::write(game_dir.join("gog.ico"), "fake").unwrap();

    let sigs_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/signatures");
    let sigs = signatures::load_signatures_from_dir(&sigs_dir).unwrap();

    let result = scanner::scan_directory(games_dir.path(), &sigs, 4).unwrap();
    let all_games: Vec<_> = result
        .high_confidence
        .iter()
        .chain(result.needs_confirmation.iter())
        .collect();

    assert!(!all_games.is_empty(), "GOG game must be detected");
    assert_eq!(
        all_games[0].store_origin,
        fenrir_core::library::game::StoreOrigin::GOG
    );
    assert_eq!(
        all_games[0].crack_type,
        Some(fenrir_core::library::game::CrackType::GOGRip)
    );
}

// ---------------------------------------------------------------------------
// Test 12 (Fase 2): SHA-512 checksum computation and verification
// ---------------------------------------------------------------------------

#[test]
fn test_checksum_verification_roundtrip() {
    use fenrir_core::runtime::downloader::{compute_sha512, verify_sha512};

    let payload = b"fenrir fase2 checksum test payload";
    let hash = compute_sha512(payload);

    // Correct hash must verify
    assert!(verify_sha512(payload, &hash));

    // Different data must NOT verify against original hash
    assert!(!verify_sha512(b"different data", &hash));

    // Corrupt hash must NOT verify
    assert!(!verify_sha512(payload, "0000000000000000"));

    // Hash must be deterministic
    assert_eq!(compute_sha512(payload), hash);
}

// ---------------------------------------------------------------------------
// Test 13 (Fase 2): GitHub Release JSON parsing
// ---------------------------------------------------------------------------

#[test]
fn test_github_release_parsing() {
    use fenrir_core::runtime::github::GitHubRelease;

    let json = r#"[{
        "tag_name": "GE-Proton9-20",
        "name": "GE-Proton9-20",
        "assets": [
            {
                "name": "GE-Proton9-20.tar.gz",
                "browser_download_url": "https://example.com/GE-Proton9-20.tar.gz",
                "size": 419430400
            },
            {
                "name": "GE-Proton9-20.sha512sum",
                "browser_download_url": "https://example.com/GE-Proton9-20.sha512sum",
                "size": 128
            }
        ]
    }]"#;

    let releases: Vec<GitHubRelease> = serde_json::from_str(json).unwrap();
    assert_eq!(releases.len(), 1);

    let release = &releases[0];
    assert_eq!(release.tag_name, "GE-Proton9-20");
    assert_eq!(release.assets.len(), 2);

    let tarball = release.find_tarball();
    assert!(tarball.is_some(), "tarball asset must be found");
    assert!(tarball.unwrap().name.ends_with(".tar.gz"));
    assert_eq!(tarball.unwrap().size, 419_430_400);

    let checksum = release.find_checksum();
    assert!(checksum.is_some(), "checksum asset must be found");
    assert!(checksum.unwrap().name.ends_with(".sha512sum"));
}

// ---------------------------------------------------------------------------
// Test 15 (Sprint 1): Exe in subfolder is promoted to the correct game root
// ---------------------------------------------------------------------------

#[test]
fn test_scanner_nested_exe_finds_correct_root() {
    let games_dir = tempdir().unwrap();
    let game_root = games_dir.path().join("Elden Ring");
    let game_subdir = game_root.join("Game");
    fs::create_dir_all(&game_subdir).unwrap();
    fs::write(game_root.join("steam_api64.dll"), "fake").unwrap();
    fs::write(game_root.join("steam_appid.txt"), "1245620").unwrap();
    fs::write(game_subdir.join("eldenring.exe"), "fake").unwrap();

    let sig_toml = r#"
[steam_generic_64]
name = "Steam Generic Crack (64-bit)"
store = "Steam"
required_files = ["steam_api64.dll"]
optional_files = ["steam_appid.txt"]
confidence_boost = []
"#;
    let sigs = signatures::parse_signatures_from_str(sig_toml).unwrap();
    let result = scanner::scan_directory(games_dir.path(), &sigs, 6).unwrap();

    let all: Vec<_> = result
        .high_confidence
        .iter()
        .chain(result.needs_confirmation.iter())
        .collect();
    assert_eq!(all.len(), 1, "should find exactly one game");
    assert_eq!(
        all[0].path, game_root,
        "root should be 'Elden Ring/', not 'Game/'"
    );
    assert!(all[0].confidence >= 30);
}

// ---------------------------------------------------------------------------
// Test 16 (Sprint 1): System dirs inside a game folder produce no extra candidates
// ---------------------------------------------------------------------------

#[test]
fn test_scanner_no_false_positives_from_system_dirs() {
    let games_dir = tempdir().unwrap();
    let game_root = games_dir.path().join("Some Game");
    fs::create_dir_all(&game_root).unwrap();
    fs::write(game_root.join("steam_api.dll"), "fake").unwrap();
    fs::write(game_root.join("game.exe"), "fake").unwrap();

    let common_redist = game_root.join("_CommonRedist").join("vcredist");
    fs::create_dir_all(&common_redist).unwrap();
    fs::write(common_redist.join("vcredist_x64.exe"), "fake").unwrap();

    let dx = game_root.join("DirectX");
    fs::create_dir_all(&dx).unwrap();
    fs::write(dx.join("dxsetup.exe"), "fake").unwrap();

    let sig_toml = r#"
[steam_generic]
name = "Steam Generic"
store = "Steam"
required_files = ["steam_api.dll"]
optional_files = []
confidence_boost = []
"#;
    let sigs = signatures::parse_signatures_from_str(sig_toml).unwrap();
    let result = scanner::scan_directory(games_dir.path(), &sigs, 6).unwrap();

    // Only one candidate total — the game itself, not any system subdir
    assert_eq!(
        result.total_candidates, 1,
        "should not create candidates for system dirs"
    );
    let all: Vec<_> = result
        .high_confidence
        .iter()
        .chain(result.needs_confirmation.iter())
        .collect();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].path, game_root);
}

// ---------------------------------------------------------------------------
// Test 14 (Fase 2): All crack types have a corresponding Wine profile on disk
// ---------------------------------------------------------------------------

#[test]
fn test_all_crack_types_have_profiles() {
    let profiles_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/profiles");
    let profiles = load_profiles_from_dir(&profiles_dir).unwrap();

    let required = [
        "steam_generic",
        "onlinefix",
        "fitgirl",
        "dodi",
        "scene",
        "gog",
    ];
    for name in &required {
        assert!(
            profiles.contains_key(*name),
            "missing Wine profile for crack type: {}",
            name
        );
    }
}

// ---------------------------------------------------------------------------
// Test 15 (Sprint 1 consolidation): scan of an Unreal-style layout classifies
// the game at its real root, not the Binaries/Win64 subfolder.
// ---------------------------------------------------------------------------

#[test]
fn test_scan_unreal_layout_uses_real_root() {
    let games_dir = tempdir().unwrap();
    let game_dir = games_dir.path().join("EldenRing");
    let bin_dir = game_dir.join("Game").join("Binaries").join("Win64");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::write(bin_dir.join("eldenring.exe"), "fake").unwrap();
    fs::write(game_dir.join("steam_api64.dll"), "fake").unwrap();
    fs::write(game_dir.join("steam_appid.txt"), "1245620").unwrap();

    let sigs_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/signatures");
    let sigs = signatures::load_signatures_from_dir(&sigs_dir).unwrap();

    let result = scanner::scan_directory(games_dir.path(), &sigs, 8).unwrap();
    let all: Vec<_> = result
        .high_confidence
        .iter()
        .chain(result.needs_confirmation.iter())
        .collect();

    assert_eq!(all.len(), 1, "exactly one classified game at the real root");
    assert_eq!(all[0].path, game_dir);
    assert_eq!(all[0].store_origin, StoreOrigin::Steam);
    // steam_api64 (required=30) + steam_appid (optional=15) = 45 → needs_confirmation
    assert!(all[0].confidence >= 30);
}

// ---------------------------------------------------------------------------
// Test 16 (Sprint 1 consolidation): system-level redistributable directories
// do not generate spurious candidates.
// ---------------------------------------------------------------------------

#[test]
fn test_scan_ignores_system_dirs_no_false_positives() {
    let games_dir = tempdir().unwrap();

    // A real game
    let game = games_dir.path().join("RealGame");
    fs::create_dir(&game).unwrap();
    fs::write(game.join("game.exe"), "fake").unwrap();
    fs::write(game.join("steam_api.dll"), "fake").unwrap();

    // Redist noise next to the game
    for noise in &["_CommonRedist", "DirectX", "_Redist", "vcredist"] {
        let d = games_dir.path().join(noise);
        fs::create_dir(&d).unwrap();
        fs::write(d.join("installer.exe"), "fake").unwrap();
    }

    // Engine/ subdir inside the game (common for Unreal titles) with a helper exe
    let engine = game.join("Engine").join("Binaries").join("Win64");
    fs::create_dir_all(&engine).unwrap();
    fs::write(engine.join("CrashReportClient.exe"), "fake").unwrap();

    let sigs_dir =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/signatures");
    let sigs = signatures::load_signatures_from_dir(&sigs_dir).unwrap();

    let result = scanner::scan_directory(games_dir.path(), &sigs, 8).unwrap();
    let all: Vec<_> = result
        .high_confidence
        .iter()
        .chain(result.needs_confirmation.iter())
        .collect();

    assert_eq!(
        all.len(),
        1,
        "only the real game must be classified — found {}",
        all.iter()
            .map(|g| g.title.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    assert_eq!(all[0].path, game);
}

// ---------------------------------------------------------------------------
// Test 17 (Sprint 1): Full pipeline — scan → DB → configure state → launch cmd
// ---------------------------------------------------------------------------

#[test]
fn test_full_pipeline_scan_to_launch_command() {
    use fenrir_core::launcher::runner::{build_launch_command, LaunchConfig};
    use std::collections::HashMap;

    // 1. Fake game on disk
    let games_dir = tempdir().unwrap();
    let game_dir = games_dir.path().join("Hollow Knight");
    fs::create_dir(&game_dir).unwrap();
    fs::write(game_dir.join("hollow_knight.exe"), "fake").unwrap();
    fs::write(game_dir.join("steam_api.dll"), "fake").unwrap();
    fs::write(game_dir.join("steam_api64.dll"), "fake").unwrap();
    fs::write(game_dir.join("steam_appid.txt"), "367520").unwrap();

    let sigs = signatures::parse_signatures_from_str(
        r#"
[steam_generic]
name = "Steam Generic Crack"
store = "Steam"
required_files = ["steam_api.dll"]
optional_files = ["steam_api64.dll", "steam_appid.txt"]
confidence_boost = []
"#,
    )
    .unwrap();

    // 2. Scan
    let result = scanner::scan_directory(games_dir.path(), &sigs, 4).unwrap();
    assert_eq!(result.high_confidence.len(), 1);
    let classified = &result.high_confidence[0];
    assert_eq!(classified.title, "Hollow Knight");

    // 3. Persist as Detected
    let db = Database::open_in_memory().unwrap();
    let game_id = uuid::Uuid::new_v4();
    let exe = classified.exe_files.first().cloned().unwrap();
    let mut game = Game {
        id: game_id,
        title: classified.title.clone(),
        executable: exe.clone(),
        install_dir: classified.path.clone(),
        store_origin: classified.store_origin,
        crack_type: classified.crack_type,
        prefix_path: PathBuf::new(),
        runtime_id: None,
        status: GameStatus::Detected,
        play_time: 0,
        last_played: None,
        added_at: chrono::Utc::now(),
        user_overrides: None,
    };
    db.insert_game(&game).unwrap();

    // 4. Simulate configure
    let prefix_dir = tempdir().unwrap();
    let prefix_path = fenrir_core::prefix::prefix_path_for_game(prefix_dir.path(), game_id);
    game.prefix_path = prefix_path.clone();
    game.runtime_id = Some("GE-Proton9-20".to_string());
    game.status = GameStatus::Configured;
    db.update_game(&game).unwrap();

    // 5. Build launch command (Wine path)
    let launch_cmd = build_launch_command(&LaunchConfig {
        executable: game.executable.clone(),
        wine_binary: PathBuf::from("/usr/bin/wine"),
        prefix_path: game.prefix_path.clone(),
        env_vars: HashMap::new(),
        is_proton: false,
        proton_path: None,
        steam_app_id: None,
    });

    // 6. Verify the full chain produced a valid command
    let fetched = db.get_game(game_id).unwrap().unwrap();
    assert_eq!(fetched.status, GameStatus::Configured);
    assert_eq!(fetched.title, "Hollow Knight");
    assert_eq!(launch_cmd.program, "/usr/bin/wine");
    assert_eq!(launch_cmd.args, vec![exe.to_string_lossy().as_ref()]);
    assert_eq!(
        launch_cmd.env.get("WINEPREFIX").unwrap(),
        &prefix_path.to_string_lossy().to_string()
    );
    assert_eq!(launch_cmd.working_dir, game_dir);
}

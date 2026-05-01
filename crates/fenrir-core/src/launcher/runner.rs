use crate::error::LauncherError;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use tracing::info;

pub struct LaunchConfig {
    pub executable: PathBuf,
    pub wine_binary: PathBuf,
    pub prefix_path: PathBuf,
    pub env_vars: HashMap<String, String>,
    pub is_proton: bool,
    pub proton_path: Option<PathBuf>,
    pub steam_app_id: Option<String>,
}

pub struct PreparedCommand {
    pub program: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: PathBuf,
}

/// Reads the Steam AppID for a game from its install directory.
/// Checks OnlineFix.ini (FakeAppId) first, then steam_appid.txt.
/// Returns None if neither is found or readable.
pub fn read_steam_app_id(install_dir: &std::path::Path) -> Option<String> {
    // OnlineFix.ini takes priority: FakeAppId is the Spacewar ID used for Steam IPC
    if let Some(v) = read_fake_app_id_from_ini(&install_dir.join("OnlineFix.ini")) {
        return Some(v);
    }

    // Unreal Engine layout: <install_dir>/<game>/Binaries/Win64/OnlineFix.ini
    if let Ok(entries) = std::fs::read_dir(install_dir) {
        for entry in entries.flatten() {
            let sub = entry.path();
            if !sub.is_dir() {
                continue;
            }
            let ini = sub.join("Binaries").join("Win64").join("OnlineFix.ini");
            if let Some(v) = read_fake_app_id_from_ini(&ini) {
                return Some(v);
            }
        }
    }

    // Fallback: steam_appid.txt
    let appid_txt = install_dir.join("steam_appid.txt");
    if appid_txt.exists() {
        if let Ok(content) = std::fs::read_to_string(&appid_txt) {
            let v = content.trim().to_string();
            if !v.is_empty() {
                return Some(v);
            }
        }
    }

    None
}

fn read_fake_app_id_from_ini(path: &std::path::Path) -> Option<String> {
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some(value) = line.trim().strip_prefix("FakeAppId=") {
            let v = value.trim().to_string();
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

/// Builds the LD_PRELOAD value for Steam overlay injection.
/// Looks for gameoverlayrenderer.so in ubuntu12_32/ and ubuntu12_64/ under steam_path.
/// Order matches Valve's own launch scripts: 32-bit first, then 64-bit.
/// Appends to existing LD_PRELOAD value (colon-separated). Returns None if no .so found.
pub fn build_overlay_ld_preload(steam_path: &std::path::Path, existing: &str) -> Option<String> {
    let candidates = [
        steam_path.join("ubuntu12_32/gameoverlayrenderer.so"),
        steam_path.join("ubuntu12_64/gameoverlayrenderer.so"),
    ];
    let paths: Vec<String> = candidates
        .iter()
        .filter(|p| p.exists())
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    if paths.is_empty() {
        return None;
    }

    let existing_trimmed = existing.trim_end_matches(':');
    Some(if existing_trimmed.is_empty() {
        paths.join(":")
    } else {
        format!("{}:{}", existing_trimmed, paths.join(":"))
    })
}

/// Build the launch command without executing it.
pub fn build_launch_command(config: &LaunchConfig) -> PreparedCommand {
    let mut env = config.env_vars.clone();
    let working_dir = config
        .executable
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    // Steam AppID env vars — both Wine and Proton need these for IPC connection.
    // SteamOverlayGameId is the specific variable gameoverlayrenderer.so uses to
    // register the game process for invite/lobby-join callback routing; without
    // it the overlay renders but "accept invite" clicks are never delivered to
    // the process. ENABLE_VK_LAYER_VALVE_steam_overlay_1 activates the Valve
    // Vulkan overlay layer for DXVK/VKD3D-rendered games (Unreal, Unity, etc.)
    // — the GL/DX-only hooks in gameoverlayrenderer.so don't intercept Vulkan.
    if let Some(ref app_id) = config.steam_app_id {
        env.insert("SteamGameId".to_string(), app_id.clone());
        env.insert("SteamAppId".to_string(), app_id.clone());
        env.insert("SteamOverlayGameId".to_string(), app_id.clone());
        env.insert(
            "ENABLE_VK_LAYER_VALVE_steam_overlay_1".to_string(),
            "1".to_string(),
        );
    }

    let steam_install_dir = crate::runtime::discovery::find_steam_install_dir();

    // Steam overlay injection: LD_PRELOAD with gameoverlayrenderer.so
    if config.steam_app_id.is_some() {
        if let Some(ref steam_path) = steam_install_dir {
            let existing = env.get("LD_PRELOAD").cloned().unwrap_or_default();
            if let Some(ld_preload) = build_overlay_ld_preload(steam_path, &existing) {
                env.insert("LD_PRELOAD".to_string(), ld_preload);
            }
        } else if !config.is_proton {
            tracing::warn!("Steam installation not found, Steam overlay will not be available");
        }
    }

    if config.is_proton {
        let steam_path = steam_install_dir.unwrap_or_else(|| {
            tracing::warn!(
                "Steam installation not found, falling back to prefix path for \
                     STEAM_COMPAT_CLIENT_INSTALL_PATH"
            );
            config.prefix_path.clone()
        });

        env.insert(
            "STEAM_COMPAT_DATA_PATH".to_string(),
            config.prefix_path.to_string_lossy().to_string(),
        );
        env.insert(
            "STEAM_COMPAT_CLIENT_INSTALL_PATH".to_string(),
            steam_path.to_string_lossy().to_string(),
        );

        if let Some(ref app_id) = config.steam_app_id {
            env.insert("STEAM_COMPAT_APP_ID".to_string(), app_id.clone());
        }

        // For Steam-integrated games (steam_app_id set), wrap the Proton
        // invocation in SteamLinuxRuntime_sniper when available. The sniper
        // container provides the Steam IPC environment required for overlay
        // invite-join callbacks to reach the game process. Without it, raw
        // Proton runs outside the container and lobby-join requests are
        // silently dropped even though the overlay itself renders.
        let sniper = if config.steam_app_id.is_some() {
            crate::runtime::discovery::find_steam_linux_runtime_sniper()
        } else {
            None
        };

        if let Some(sniper_dir) = sniper {
            PreparedCommand {
                program: sniper_dir.join("run").to_string_lossy().to_string(),
                args: vec![
                    "--".to_string(),
                    config.wine_binary.to_string_lossy().to_string(),
                    "run".to_string(),
                    config.executable.to_string_lossy().to_string(),
                ],
                env,
                working_dir,
            }
        } else {
            PreparedCommand {
                program: config.wine_binary.to_string_lossy().to_string(),
                args: vec![
                    "run".to_string(),
                    config.executable.to_string_lossy().to_string(),
                ],
                env,
                working_dir,
            }
        }
    } else {
        env.insert(
            "WINEPREFIX".to_string(),
            config.prefix_path.to_string_lossy().to_string(),
        );

        PreparedCommand {
            program: config.wine_binary.to_string_lossy().to_string(),
            args: vec![config.executable.to_string_lossy().to_string()],
            env,
            working_dir,
        }
    }
}

/// Execute the launch command and return the child process.
pub fn launch(config: &LaunchConfig) -> Result<Child, LauncherError> {
    if !config.executable.exists() {
        return Err(LauncherError::ExeNotFound(config.executable.clone()));
    }

    let cmd = build_launch_command(config);

    info!("launching: {} {:?}", cmd.program, cmd.args);

    let child = Command::new(&cmd.program)
        .args(&cmd.args)
        .envs(&cmd.env)
        .current_dir(&cmd.working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;

    Ok(child)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_wine_command() {
        let cmd = build_launch_command(&LaunchConfig {
            executable: PathBuf::from("/games/test/game.exe"),
            wine_binary: PathBuf::from("/usr/bin/wine"),
            prefix_path: PathBuf::from("/tmp/prefix"),
            env_vars: HashMap::from([("WINEPREFIX".to_string(), "/tmp/prefix".to_string())]),
            is_proton: false,
            proton_path: None,
            steam_app_id: None,
        });
        assert_eq!(cmd.program, "/usr/bin/wine");
        assert_eq!(cmd.args, vec!["/games/test/game.exe"]);
        assert_eq!(cmd.env.get("WINEPREFIX").unwrap(), "/tmp/prefix");
        assert_eq!(cmd.working_dir, PathBuf::from("/games/test"));
    }

    #[test]
    fn test_build_proton_command() {
        let cmd = build_launch_command(&LaunchConfig {
            executable: PathBuf::from("/games/test/game.exe"),
            wine_binary: PathBuf::from("/runtimes/GE-Proton9-20/proton"),
            prefix_path: PathBuf::from("/tmp/prefix"),
            env_vars: HashMap::new(),
            is_proton: true,
            proton_path: Some(PathBuf::from("/runtimes/GE-Proton9-20")),
            steam_app_id: None,
        });
        assert_eq!(cmd.program, "/runtimes/GE-Proton9-20/proton");
        assert_eq!(cmd.args, vec!["run", "/games/test/game.exe"]);
        assert!(cmd.env.contains_key("STEAM_COMPAT_DATA_PATH"));
        assert!(cmd.env.contains_key("STEAM_COMPAT_CLIENT_INSTALL_PATH"));
    }

    #[test]
    fn test_working_dir_from_executable() {
        let cmd = build_launch_command(&LaunchConfig {
            executable: PathBuf::from("/mnt/games/cyberpunk/bin/x64/Cyberpunk2077.exe"),
            wine_binary: PathBuf::from("/usr/bin/wine"),
            prefix_path: PathBuf::from("/tmp/prefix"),
            env_vars: HashMap::new(),
            is_proton: false,
            proton_path: None,
            steam_app_id: None,
        });
        assert_eq!(
            cmd.working_dir,
            PathBuf::from("/mnt/games/cyberpunk/bin/x64")
        );
    }

    #[test]
    fn test_read_steam_app_id_onlinefix() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("OnlineFix.ini"),
            "[Main]\nRealAppId=3643170\nFakeAppId=480\n",
        )
        .unwrap();
        assert_eq!(read_steam_app_id(dir.path()), Some("480".to_string()));
    }

    #[test]
    fn test_read_steam_app_id_txt() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("steam_appid.txt"), "945360\n").unwrap();
        assert_eq!(read_steam_app_id(dir.path()), Some("945360".to_string()));
    }

    #[test]
    fn test_read_steam_app_id_priority() {
        // OnlineFix.ini must win over steam_appid.txt
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("OnlineFix.ini"), "[Main]\nFakeAppId=480\n").unwrap();
        std::fs::write(dir.path().join("steam_appid.txt"), "945360\n").unwrap();
        assert_eq!(read_steam_app_id(dir.path()), Some("480".to_string()));
    }

    #[test]
    fn test_read_steam_app_id_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_steam_app_id(dir.path()), None);
    }

    #[test]
    fn test_read_steam_app_id_unreal_deep_scan() {
        // Unreal layout: <install_dir>/<game>/Binaries/Win64/OnlineFix.ini
        let dir = tempfile::tempdir().unwrap();
        let ini_path = dir
            .path()
            .join("AnomalyCompany")
            .join("Binaries")
            .join("Win64");
        std::fs::create_dir_all(&ini_path).unwrap();
        std::fs::write(
            ini_path.join("OnlineFix.ini"),
            "[Main]\nRealAppId=3643170\nFakeAppId=480\n",
        )
        .unwrap();
        assert_eq!(read_steam_app_id(dir.path()), Some("480".to_string()));
    }

    #[test]
    fn test_read_steam_app_id_root_wins_over_unreal_deep() {
        // Root-level OnlineFix.ini must take priority over Unreal deep path
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("OnlineFix.ini"), "[Main]\nFakeAppId=999\n").unwrap();
        let ini_path = dir.path().join("GameDir").join("Binaries").join("Win64");
        std::fs::create_dir_all(&ini_path).unwrap();
        std::fs::write(ini_path.join("OnlineFix.ini"), "[Main]\nFakeAppId=480\n").unwrap();
        assert_eq!(read_steam_app_id(dir.path()), Some("999".to_string()));
    }

    #[test]
    fn test_build_command_steam_vars_wine() {
        let cmd = build_launch_command(&LaunchConfig {
            executable: PathBuf::from("/games/test/game.exe"),
            wine_binary: PathBuf::from("/usr/bin/wine"),
            prefix_path: PathBuf::from("/tmp/prefix"),
            env_vars: HashMap::new(),
            is_proton: false,
            proton_path: None,
            steam_app_id: Some("480".to_string()),
        });
        assert_eq!(cmd.env.get("SteamGameId").unwrap(), "480");
        assert_eq!(cmd.env.get("SteamAppId").unwrap(), "480");
        assert_eq!(cmd.env.get("SteamOverlayGameId").unwrap(), "480");
        assert_eq!(
            cmd.env
                .get("ENABLE_VK_LAYER_VALVE_steam_overlay_1")
                .unwrap(),
            "1"
        );
        // STEAM_COMPAT_APP_ID must NOT be set for Wine
        assert!(!cmd.env.contains_key("STEAM_COMPAT_APP_ID"));
    }

    #[test]
    fn test_build_command_steam_vars_proton() {
        let cmd = build_launch_command(&LaunchConfig {
            executable: PathBuf::from("/games/test/game.exe"),
            wine_binary: PathBuf::from("/runtimes/GE-Proton9-20/proton"),
            prefix_path: PathBuf::from("/tmp/prefix"),
            env_vars: HashMap::new(),
            is_proton: true,
            proton_path: Some(PathBuf::from("/runtimes/GE-Proton9-20")),
            steam_app_id: Some("480".to_string()),
        });
        assert_eq!(cmd.env.get("SteamGameId").unwrap(), "480");
        assert_eq!(cmd.env.get("SteamAppId").unwrap(), "480");
        assert_eq!(cmd.env.get("SteamOverlayGameId").unwrap(), "480");
        assert_eq!(
            cmd.env
                .get("ENABLE_VK_LAYER_VALVE_steam_overlay_1")
                .unwrap(),
            "1"
        );
        assert_eq!(cmd.env.get("STEAM_COMPAT_APP_ID").unwrap(), "480");
        assert!(cmd.env.contains_key("STEAM_COMPAT_CLIENT_INSTALL_PATH"));
    }

    #[test]
    fn test_build_command_no_steam_app_id() {
        let cmd = build_launch_command(&LaunchConfig {
            executable: PathBuf::from("/games/test/game.exe"),
            wine_binary: PathBuf::from("/usr/bin/wine"),
            prefix_path: PathBuf::from("/tmp/prefix"),
            env_vars: HashMap::new(),
            is_proton: false,
            proton_path: None,
            steam_app_id: None,
        });
        assert!(!cmd.env.contains_key("SteamGameId"));
        assert!(!cmd.env.contains_key("SteamAppId"));
        assert!(!cmd.env.contains_key("SteamOverlayGameId"));
        assert!(!cmd
            .env
            .contains_key("ENABLE_VK_LAYER_VALVE_steam_overlay_1"));
    }

    #[test]
    fn test_build_overlay_ld_preload_both_so() {
        let dir = tempfile::tempdir().unwrap();
        let so64 = dir.path().join("ubuntu12_64");
        let so32 = dir.path().join("ubuntu12_32");
        std::fs::create_dir_all(&so64).unwrap();
        std::fs::create_dir_all(&so32).unwrap();
        std::fs::write(so64.join("gameoverlayrenderer.so"), "").unwrap();
        std::fs::write(so32.join("gameoverlayrenderer.so"), "").unwrap();

        let result = build_overlay_ld_preload(dir.path(), "");
        assert!(result.is_some());
        let val = result.unwrap();
        assert!(val.contains("ubuntu12_64/gameoverlayrenderer.so"));
        assert!(val.contains("ubuntu12_32/gameoverlayrenderer.so"));
        let idx64 = val.find("ubuntu12_64").unwrap();
        let idx32 = val.find("ubuntu12_32").unwrap();
        assert!(
            idx32 < idx64,
            "32-bit overlay must precede 64-bit (Valve convention)"
        );
    }

    #[test]
    fn test_build_overlay_ld_preload_appends_existing() {
        let dir = tempfile::tempdir().unwrap();
        let so64 = dir.path().join("ubuntu12_64");
        std::fs::create_dir_all(&so64).unwrap();
        std::fs::write(so64.join("gameoverlayrenderer.so"), "").unwrap();

        let result = build_overlay_ld_preload(dir.path(), "/usr/lib/libfoo.so");
        assert!(result.is_some());
        let val = result.unwrap();
        assert!(val.starts_with("/usr/lib/libfoo.so:"));
        assert!(val.contains("ubuntu12_64/gameoverlayrenderer.so"));
    }

    #[test]
    fn test_build_overlay_ld_preload_no_so() {
        let dir = tempfile::tempdir().unwrap();
        // no .so files created
        let result = build_overlay_ld_preload(dir.path(), "");
        assert!(result.is_none());
    }

    #[test]
    fn test_build_overlay_ld_preload_only_one_so() {
        let dir = tempfile::tempdir().unwrap();
        let so64 = dir.path().join("ubuntu12_64");
        std::fs::create_dir_all(&so64).unwrap();
        std::fs::write(so64.join("gameoverlayrenderer.so"), "").unwrap();
        // ubuntu12_32 not created

        let result = build_overlay_ld_preload(dir.path(), "");
        assert!(result.is_some());
        let val = result.unwrap();
        assert!(val.contains("ubuntu12_64/gameoverlayrenderer.so"));
        assert!(!val.contains("ubuntu12_32/gameoverlayrenderer.so"));
    }

    #[test]
    fn test_build_overlay_ld_preload_existing_trailing_colon() {
        let dir = tempfile::tempdir().unwrap();
        let so64 = dir.path().join("ubuntu12_64");
        std::fs::create_dir_all(&so64).unwrap();
        std::fs::write(so64.join("gameoverlayrenderer.so"), "").unwrap();

        let result = build_overlay_ld_preload(dir.path(), "/usr/lib/libfoo.so:");
        assert!(result.is_some());
        let val = result.unwrap();
        assert!(
            !val.contains("::"),
            "must not produce double colon in LD_PRELOAD"
        );
        assert!(val.starts_with("/usr/lib/libfoo.so:"));
    }

    #[test]
    fn test_build_command_no_overlay_without_steam_app_id() {
        // When steam_app_id is None, LD_PRELOAD must not be set by the overlay logic
        let cmd = build_launch_command(&LaunchConfig {
            executable: PathBuf::from("/games/test/game.exe"),
            wine_binary: PathBuf::from("/usr/bin/wine"),
            prefix_path: PathBuf::from("/tmp/prefix"),
            env_vars: HashMap::new(),
            is_proton: false,
            proton_path: None,
            steam_app_id: None,
        });
        assert!(!cmd.env.contains_key("LD_PRELOAD"));
    }

    #[test]
    fn test_build_command_overlay_preserves_existing_ld_preload() {
        // When steam_app_id is Some and a pre-existing LD_PRELOAD is set,
        // the result must either keep the original value (if Steam not installed)
        // or append to it (if Steam is installed). Either way the original value must survive.
        let existing = "/usr/lib/libfoo.so".to_string();
        let cmd = build_launch_command(&LaunchConfig {
            executable: PathBuf::from("/games/test/game.exe"),
            wine_binary: PathBuf::from("/usr/bin/wine"),
            prefix_path: PathBuf::from("/tmp/prefix"),
            env_vars: HashMap::from([("LD_PRELOAD".to_string(), existing.clone())]),
            is_proton: false,
            proton_path: None,
            steam_app_id: Some("480".to_string()),
        });
        let ld = cmd
            .env
            .get("LD_PRELOAD")
            .expect("LD_PRELOAD must be present (set via env_vars)");
        assert!(
            ld.contains(&existing),
            "original LD_PRELOAD value must be preserved"
        );
    }
}

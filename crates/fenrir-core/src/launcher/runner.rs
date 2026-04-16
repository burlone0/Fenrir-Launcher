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
    let onlinefix_ini = install_dir.join("OnlineFix.ini");
    if onlinefix_ini.exists() {
        if let Ok(content) = std::fs::read_to_string(&onlinefix_ini) {
            for line in content.lines() {
                if let Some(value) = line.trim().strip_prefix("FakeAppId=") {
                    let v = value.trim().to_string();
                    if !v.is_empty() {
                        return Some(v);
                    }
                }
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

/// Build the launch command without executing it.
pub fn build_launch_command(config: &LaunchConfig) -> PreparedCommand {
    let mut env = config.env_vars.clone();
    let working_dir = config
        .executable
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    if config.is_proton {
        env.insert(
            "STEAM_COMPAT_DATA_PATH".to_string(),
            config.prefix_path.to_string_lossy().to_string(),
        );
        env.insert(
            "STEAM_COMPAT_CLIENT_INSTALL_PATH".to_string(),
            config.prefix_path.to_string_lossy().to_string(),
        );

        PreparedCommand {
            program: config.wine_binary.to_string_lossy().to_string(),
            args: vec![
                "run".to_string(),
                config.executable.to_string_lossy().to_string(),
            ],
            env,
            working_dir,
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
}

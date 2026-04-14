use crate::error::PrefixError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info};
use uuid::Uuid;

/// Generate the prefix path for a game.
pub fn prefix_path_for_game(prefix_dir: &Path, game_id: Uuid) -> PathBuf {
    prefix_dir.join(game_id.to_string())
}

/// Create and initialize a WINEPREFIX.
pub fn create_prefix(
    prefix_path: &Path,
    wine_binary: &Path,
    is_proton: bool,
) -> Result<(), PrefixError> {
    std::fs::create_dir_all(prefix_path).map_err(|e| PrefixError::Directory(e.to_string()))?;

    info!("initializing prefix at {}", prefix_path.display());

    // For Proton runtimes the `proton` wrapper script requires Steam environment variables
    // that are unavailable during prefix creation. Use the internal Wine binary instead,
    // which lives at <proton_dir>/files/bin/wine and works identically for initialization.
    let effective_wine = if is_proton {
        wine_binary
            .parent()
            .map(|proton_dir| proton_dir.join("files/bin/wine"))
            .filter(|p| p.exists())
            .unwrap_or_else(|| wine_binary.to_path_buf())
    } else {
        wine_binary.to_path_buf()
    };

    let output = Command::new(&effective_wine)
        .arg("wineboot")
        .arg("--init")
        .env("WINEPREFIX", prefix_path)
        .env("WINEDEBUG", "-all")
        .output()
        .map_err(|e| PrefixError::WinebootFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PrefixError::WinebootFailed(stderr.to_string()));
    }

    debug!("prefix initialized: {}", prefix_path.display());
    Ok(())
}

/// Build base Wine environment variables for a prefix.
pub fn build_wine_env(prefix_path: &Path, esync: bool, fsync: bool) -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert(
        "WINEPREFIX".to_string(),
        prefix_path.to_string_lossy().to_string(),
    );
    env.insert("WINEDEBUG".to_string(), "-all".to_string());

    if esync {
        env.insert("WINEESYNC".to_string(), "1".to_string());
    }
    if fsync {
        env.insert("WINEFSYNC".to_string(), "1".to_string());
    }

    env
}

/// Apply DLL overrides to the Wine prefix registry.
pub fn set_dll_overrides(
    prefix_path: &Path,
    wine_binary: &Path,
    overrides: &[String],
) -> Result<(), PrefixError> {
    if overrides.is_empty() {
        return Ok(());
    }

    debug!("setting DLL overrides: {:?}", overrides);

    let reg_content = format!(
        "REGEDIT4\n\n[HKEY_CURRENT_USER\\Software\\Wine\\DllOverrides]\n{}",
        overrides
            .iter()
            .map(|o| {
                let parts: Vec<&str> = o.splitn(2, '=').collect();
                if parts.len() == 2 {
                    format!("\"{}\"=\"{}\"", parts[0], parts[1])
                } else {
                    format!("\"{}\"=\"native\"", parts[0])
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    );

    let reg_file = prefix_path.join("dll_overrides.reg");
    std::fs::write(&reg_file, &reg_content).map_err(PrefixError::Io)?;

    let output = Command::new(wine_binary)
        .arg("regedit")
        .arg(&reg_file)
        .env("WINEPREFIX", prefix_path)
        .env("WINEDEBUG", "-all")
        .output()
        .map_err(|e| PrefixError::WinebootFailed(e.to_string()))?;

    let _ = std::fs::remove_file(&reg_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PrefixError::WinebootFailed(format!(
            "regedit failed: {}",
            stderr
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_path_creation() {
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        let path = prefix_path_for_game(dir.path(), id);
        assert!(path.ends_with(id.to_string()));
        assert_eq!(path.parent().unwrap(), dir.path());
    }

    #[test]
    fn test_build_wine_env_all_features() {
        let prefix = PathBuf::from("/tmp/test-prefix");
        let env = build_wine_env(&prefix, true, true);
        assert_eq!(env.get("WINEPREFIX").unwrap(), "/tmp/test-prefix");
        assert_eq!(env.get("WINEESYNC").unwrap(), "1");
        assert_eq!(env.get("WINEFSYNC").unwrap(), "1");
        assert_eq!(env.get("WINEDEBUG").unwrap(), "-all");
    }

    #[test]
    fn test_build_wine_env_no_sync() {
        let prefix = PathBuf::from("/tmp/test-prefix");
        let env = build_wine_env(&prefix, false, false);
        assert!(!env.contains_key("WINEESYNC"));
        assert!(!env.contains_key("WINEFSYNC"));
    }

    #[test]
    fn test_build_wine_env_mixed_sync() {
        let prefix = PathBuf::from("/tmp/test-prefix");
        let env = build_wine_env(&prefix, true, false);
        assert_eq!(env.get("WINEESYNC").unwrap(), "1");
        assert!(!env.contains_key("WINEFSYNC"));
    }

    // create_prefix with is_proton=true should fall back to the wine_binary itself
    // when files/bin/wine does not exist (non-existent proton dir path used here).
    #[test]
    fn test_create_prefix_proton_fallback_when_no_internal_wine() {
        let dir = tempfile::tempdir().unwrap();
        let fake_proton = dir.path().join("proton");
        // No files/bin/wine exists → effective_wine falls back to wine_binary
        // create_prefix will fail because fake_proton is not an executable, but
        // the failure must come from wineboot, NOT from resolving the path.
        let result = create_prefix(dir.path(), &fake_proton, true);
        assert!(
            result.is_err(),
            "expected error because proton binary is not executable"
        );
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("wineboot failed"), "unexpected error: {msg}");
    }
}

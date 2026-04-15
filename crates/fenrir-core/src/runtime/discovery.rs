use crate::runtime::types::{Runtime, RuntimeSource, RuntimeType};
use crate::runtime::version::parse_runtime_dir_name;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Discover all available Wine/Proton runtimes on the system.
pub fn discover_all(fenrir_runtime_dir: &Path) -> Vec<Runtime> {
    let mut runtimes = Vec::new();

    // 1. Fenrir-managed runtimes
    runtimes.extend(discover_runtimes_in_dir(
        fenrir_runtime_dir,
        RuntimeSource::Downloaded,
    ));

    // 2. GE-Proton installed for Steam
    if let Some(compat) = find_steam_compat_dir() {
        runtimes.extend(discover_runtimes_in_dir(&compat, RuntimeSource::Steam));
    }

    // 3. Valve official Proton
    if let Some(common) = find_steam_common_dir() {
        runtimes.extend(discover_runtimes_in_dir(&common, RuntimeSource::Steam));
    }

    // 4. System Wine
    if let Some(rt) = check_system_wine_at("/usr/bin/wine") {
        runtimes.push(rt);
    }

    info!("discovered {} runtimes", runtimes.len());
    runtimes
}

/// Scan a directory for Wine/Proton runtime subdirectories.
pub fn discover_runtimes_in_dir(dir: &Path, source: RuntimeSource) -> Vec<Runtime> {
    let mut runtimes = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => {
            debug!("runtime directory not found: {}", dir.display());
            return runtimes;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if let Some((id, runtime_type, version)) = parse_runtime_dir_name(name) {
            runtimes.push(Runtime {
                id: id.to_string(),
                runtime_type,
                version: version.to_string(),
                path: path.clone(),
                source,
                is_default: false,
            });
            debug!("found runtime: {} at {}", id, path.display());
        }
    }

    runtimes
}

/// Check if Wine is installed at a specific path.
pub fn check_system_wine_at(path: &str) -> Option<Runtime> {
    let wine_path = PathBuf::from(path);
    if wine_path.exists() {
        Some(Runtime {
            id: "system-wine".to_string(),
            runtime_type: RuntimeType::Wine,
            version: "system".to_string(),
            path: wine_path,
            source: RuntimeSource::System,
            is_default: false,
        })
    } else {
        None
    }
}

fn find_steam_compat_dir() -> Option<PathBuf> {
    let candidates = [
        dirs::home_dir().map(|h| h.join(".steam/root/compatibilitytools.d")),
        dirs::data_dir().map(|d| d.join("Steam/compatibilitytools.d")),
    ];
    candidates.into_iter().flatten().find(|p| p.exists())
}

fn find_steam_common_dir() -> Option<PathBuf> {
    let candidates = [
        dirs::home_dir().map(|h| h.join(".steam/root/steamapps/common")),
        dirs::data_dir().map(|d| d.join("Steam/steamapps/common")),
    ];
    candidates.into_iter().flatten().find(|p| p.exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_discover_in_directory() {
        let dir = tempfile::tempdir().unwrap();

        let proton_dir = dir.path().join("GE-Proton9-20");
        fs::create_dir(&proton_dir).unwrap();
        fs::write(proton_dir.join("proton"), "#!/bin/bash").unwrap();

        let runtimes = discover_runtimes_in_dir(dir.path(), RuntimeSource::Downloaded);
        assert_eq!(runtimes.len(), 1);
        assert_eq!(runtimes[0].id, "GE-Proton9-20");
        assert_eq!(runtimes[0].runtime_type, RuntimeType::ProtonGE);
        assert_eq!(runtimes[0].source, RuntimeSource::Downloaded);
    }

    #[test]
    fn test_discover_multiple_runtimes() {
        let dir = tempfile::tempdir().unwrap();

        fs::create_dir(dir.path().join("GE-Proton9-20")).unwrap();
        fs::create_dir(dir.path().join("wine-ge-8-26")).unwrap();
        fs::create_dir(dir.path().join("Proton 9.0")).unwrap();

        let runtimes = discover_runtimes_in_dir(dir.path(), RuntimeSource::Steam);
        assert_eq!(runtimes.len(), 3);
    }

    #[test]
    fn test_discover_skips_invalid_dirs() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("random-stuff")).unwrap();
        fs::create_dir(dir.path().join("not-a-runtime")).unwrap();

        let runtimes = discover_runtimes_in_dir(dir.path(), RuntimeSource::Downloaded);
        assert!(runtimes.is_empty());
    }

    #[test]
    fn test_discover_skips_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("GE-Proton9-20"), "not a dir").unwrap();

        let runtimes = discover_runtimes_in_dir(dir.path(), RuntimeSource::Downloaded);
        assert!(runtimes.is_empty());
    }

    #[test]
    fn test_discover_nonexistent_dir() {
        let runtimes =
            discover_runtimes_in_dir(Path::new("/nonexistent/path"), RuntimeSource::Downloaded);
        assert!(runtimes.is_empty());
    }

    #[test]
    fn test_check_system_wine_nonexistent() {
        assert!(check_system_wine_at("/nonexistent/path/wine").is_none());
    }

    #[test]
    fn test_check_system_wine_existing() {
        // /usr/bin/wine is guaranteed to exist in this dev environment
        if std::path::Path::new("/usr/bin/wine").exists() {
            let rt = check_system_wine_at("/usr/bin/wine").unwrap();
            assert_eq!(rt.id, "system-wine");
            assert_eq!(rt.runtime_type, RuntimeType::Wine);
            assert_eq!(rt.source, RuntimeSource::System);
            assert!(!rt.is_default);
        }
    }

    #[test]
    fn test_discover_all_returns_at_least_system_wine() {
        // discover_all searches Fenrir runtimes dir, Steam compat dirs, and /usr/bin/wine.
        // Only assert if Wine is actually installed on this machine.
        if !std::path::Path::new("/usr/bin/wine").exists() {
            return;
        }
        let empty_dir = tempfile::tempdir().unwrap();
        let runtimes = discover_all(empty_dir.path());
        assert!(
            !runtimes.is_empty(),
            "discover_all must find at least system Wine when /usr/bin/wine exists"
        );
        assert!(runtimes.iter().any(|r| r.id == "system-wine"));
    }
}

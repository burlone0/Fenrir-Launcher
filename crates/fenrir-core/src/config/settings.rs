use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FenrirConfig {
    pub general: GeneralConfig,
    pub scan: ScanConfig,
    pub privacy: PrivacyConfig,
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub library_db: PathBuf,
    pub prefix_dir: PathBuf,
    pub runtime_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub game_dirs: Vec<PathBuf>,
    pub auto_scan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub fetch_metadata: bool,
    pub fetch_covers: bool,
    pub metadata_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    pub runtime: String,
    pub enable_dxvk: bool,
    pub enable_vkd3d: bool,
    pub esync: bool,
    pub fsync: bool,
}

impl FenrirConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("fenrir")
            .join("config.toml")
    }

    fn data_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("fenrir")
    }

    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        self.save_to(&Self::config_path())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| ConfigError::Parse(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for FenrirConfig {
    fn default() -> Self {
        let data = Self::data_dir();
        Self {
            general: GeneralConfig {
                library_db: data.join("library.db"),
                prefix_dir: data.join("prefixes"),
                runtime_dir: data.join("runtimes"),
            },
            scan: ScanConfig {
                game_dirs: Vec::new(),
                auto_scan: false,
            },
            privacy: PrivacyConfig {
                fetch_metadata: false,
                fetch_covers: false,
                metadata_source: "igdb".to_string(),
            },
            defaults: DefaultsConfig {
                runtime: "auto".to_string(),
                enable_dxvk: true,
                enable_vkd3d: false,
                esync: true,
                fsync: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_valid_paths() {
        let config = FenrirConfig::default();
        assert!(!config.general.prefix_dir.as_os_str().is_empty());
        assert!(!config.general.runtime_dir.as_os_str().is_empty());
        assert!(!config.general.library_db.as_os_str().is_empty());
    }

    #[test]
    fn test_config_roundtrip_toml() {
        let config = FenrirConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: FenrirConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.scan.auto_scan, parsed.scan.auto_scan);
        assert_eq!(config.privacy.fetch_metadata, parsed.privacy.fetch_metadata);
    }

    #[test]
    fn test_load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let config = FenrirConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&path, &toml_str).unwrap();

        let loaded = FenrirConfig::load_from(&path).unwrap();
        assert_eq!(loaded.defaults.enable_dxvk, config.defaults.enable_dxvk);
    }

    #[test]
    fn test_save_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let config = FenrirConfig::default();
        config.save_to(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_save_to_creates_nested_directories() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a").join("b").join("config.toml");
        let config = FenrirConfig::default();
        config.save_to(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_load_from_invalid_toml_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "this is not valid toml =[[[").unwrap();
        let result = FenrirConfig::load_from(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_path_is_nonempty() {
        let path = FenrirConfig::config_path();
        assert!(!path.as_os_str().is_empty());
        assert!(path.to_string_lossy().contains("fenrir"));
    }

    #[test]
    fn test_default_config_values() {
        let config = FenrirConfig::default();
        assert!(config.defaults.enable_dxvk);
        assert_eq!(config.defaults.runtime, "auto");
        assert!(!config.privacy.fetch_metadata);
        assert!(!config.privacy.fetch_covers);
        assert!(config.scan.game_dirs.is_empty());
        assert!(!config.scan.auto_scan);
    }
}

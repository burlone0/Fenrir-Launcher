use crate::error::PrefixError;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct WineProfile {
    pub profile: ProfileMeta,
    pub wine: WineConfig,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub features: FeatureConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileMeta {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WineConfig {
    pub windows_version: String,
    #[serde(default)]
    pub dll_overrides: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeatureConfig {
    pub dxvk: bool,
    pub vkd3d: bool,
    pub esync: bool,
    pub fsync: bool,
}

impl WineProfile {
    pub fn parse(content: &str) -> Result<Self, PrefixError> {
        toml::from_str(content).map_err(|e| PrefixError::Directory(e.to_string()))
    }
}

/// Load all profiles from a directory. Key = profile.name
pub fn load_profiles_from_dir(dir: &Path) -> Result<HashMap<String, WineProfile>, PrefixError> {
    let mut profiles = HashMap::new();

    let entries = std::fs::read_dir(dir)
        .map_err(|e| PrefixError::Directory(format!("cannot read profiles dir: {}", e)))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            let content = std::fs::read_to_string(&path).map_err(PrefixError::Io)?;
            let profile = WineProfile::parse(&content)?;
            profiles.insert(profile.profile.name.clone(), profile);
        }
    }

    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PROFILE: &str = r#"
[profile]
name = "test"
description = "Test profile"

[wine]
windows_version = "win10"
dll_overrides = ["steam_api=n"]

[env]
MY_VAR = "value"

[features]
dxvk = true
vkd3d = false
esync = true
fsync = true
"#;

    #[test]
    fn test_parse_profile() {
        let profile = WineProfile::parse(TEST_PROFILE).unwrap();
        assert_eq!(profile.profile.name, "test");
        assert_eq!(profile.wine.windows_version, "win10");
        assert_eq!(profile.wine.dll_overrides, vec!["steam_api=n"]);
        assert_eq!(profile.env.get("MY_VAR").unwrap(), "value");
        assert!(profile.features.dxvk);
        assert!(!profile.features.vkd3d);
    }

    #[test]
    fn test_parse_profile_empty_env() {
        let content = r#"
[profile]
name = "minimal"
description = "Minimal"

[wine]
windows_version = "win10"

[features]
dxvk = false
vkd3d = false
esync = false
fsync = false
"#;
        let profile = WineProfile::parse(content).unwrap();
        assert!(profile.env.is_empty());
        assert!(profile.wine.dll_overrides.is_empty());
    }

    #[test]
    fn test_load_profiles_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.toml"), TEST_PROFILE).unwrap();

        let profiles = load_profiles_from_dir(dir.path()).unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles["test"].profile.name, "test");
    }

    #[test]
    fn test_load_ignores_non_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.toml"), TEST_PROFILE).unwrap();
        std::fs::write(dir.path().join("readme.md"), "# nothing").unwrap();

        let profiles = load_profiles_from_dir(dir.path()).unwrap();
        assert_eq!(profiles.len(), 1);
    }
}

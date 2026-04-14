use crate::error::ScannerError;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Signature {
    pub name: String,
    pub store: Option<String>,
    pub crack_type: Option<String>,
    pub required_files: Vec<String>,
    #[serde(default)]
    pub optional_files: Vec<String>,
    #[serde(default)]
    pub confidence_boost: Vec<String>,
}

pub fn parse_signatures_from_str(content: &str) -> Result<Vec<Signature>, ScannerError> {
    let map: HashMap<String, Signature> =
        toml::from_str(content).map_err(|e| ScannerError::SignatureLoad(e.to_string()))?;
    Ok(map.into_values().collect())
}

pub fn load_signatures_from_dir(dir: &Path) -> Result<Vec<Signature>, ScannerError> {
    let mut signatures = Vec::new();

    let entries = std::fs::read_dir(dir)
        .map_err(|_| ScannerError::SignatureLoad(format!("cannot read dir: {}", dir.display())))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            let content = std::fs::read_to_string(&path)?;
            let sigs = parse_signatures_from_str(&content)?;
            signatures.extend(sigs);
        }
    }

    Ok(signatures)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TOML: &str = r#"
[steam_generic]
name = "Steam Generic Crack"
store = "Steam"
required_files = ["steam_api.dll"]
optional_files = ["steam_api64.dll"]
confidence_boost = ["steam_emu.ini"]

[onlinefix]
name = "OnlineFix"
store = "Steam"
crack_type = "OnlineFix"
required_files = ["OnlineFix.url"]
optional_files = []
confidence_boost = []
"#;

    #[test]
    fn test_parse_signatures_from_str() {
        let sigs = parse_signatures_from_str(TEST_TOML).unwrap();
        assert_eq!(sigs.len(), 2);
    }

    #[test]
    fn test_signature_fields() {
        let sigs = parse_signatures_from_str(TEST_TOML).unwrap();
        let steam = sigs
            .iter()
            .find(|s| s.name == "Steam Generic Crack")
            .unwrap();
        assert_eq!(steam.store, Some("Steam".to_string()));
        assert!(steam.required_files.contains(&"steam_api.dll".to_string()));
    }

    #[test]
    fn test_load_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.toml"), TEST_TOML).unwrap();

        let sigs = load_signatures_from_dir(dir.path()).unwrap();
        assert_eq!(sigs.len(), 2);
    }
}

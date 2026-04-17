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
    #[serde(default)]
    pub auto_add_threshold: Option<u32>,
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

    #[test]
    fn test_load_all_signature_files() {
        // CARGO_MANIFEST_DIR = crates/fenrir-core/ → ../../data/signatures = repo root
        let sig_dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/signatures");
        let sigs = load_signatures_from_dir(&sig_dir).unwrap();
        // steam (6 original + 6 new) + gog (3) + epic (2) = at least 14
        assert!(
            sigs.len() >= 14,
            "expected at least 14 signatures, got {}",
            sigs.len()
        );
        // Every signature must declare at least one required file to be useful
        for sig in &sigs {
            assert!(
                !sig.required_files.is_empty(),
                "signature '{}' has no required_files",
                sig.name
            );
        }
    }

    // OnlineFix.url is a website shortcut that users routinely delete.
    // The invariant file is OnlineFix.ini (crack config, required for execution).
    #[test]
    fn test_onlinefix_required_file_is_ini_not_url() {
        let sigs = parse_signatures_from_str(
            r#"
[onlinefix]
name = "OnlineFix"
store = "Steam"
crack_type = "OnlineFix"
required_files = ["OnlineFix.ini"]
optional_files = ["OnlineFix64.dll", "OnlineFix.url", "steamclient.dll"]
confidence_boost = ["steam_settings/"]
"#,
        )
        .unwrap();
        let sig = sigs.iter().find(|s| s.name == "OnlineFix").unwrap();
        assert!(
            sig.required_files.contains(&"OnlineFix.ini".to_string()),
            "OnlineFix.ini must be required"
        );
        assert!(
            !sig.required_files.contains(&"OnlineFix.url".to_string()),
            "OnlineFix.url must NOT be required — users delete it"
        );
        assert!(
            sig.optional_files.contains(&"OnlineFix64.dll".to_string()),
            "OnlineFix64.dll should be optional"
        );
    }

    #[test]
    fn test_auto_add_threshold_parsed_from_toml() {
        let toml = r#"
[onlinefix]
name = "OnlineFix"
store = "Steam"
crack_type = "OnlineFix"
auto_add_threshold = 30
required_files = ["OnlineFix.ini"]
optional_files = []
confidence_boost = []
"#;
        let sigs = parse_signatures_from_str(toml).unwrap();
        let sig = sigs.iter().find(|s| s.name == "OnlineFix").unwrap();
        assert_eq!(sig.auto_add_threshold, Some(30));
    }

    #[test]
    fn test_auto_add_threshold_defaults_to_none() {
        let toml = r#"
[generic]
name = "Generic"
store = "Steam"
required_files = ["steam_api.dll"]
"#;
        let sigs = parse_signatures_from_str(toml).unwrap();
        let sig = sigs.iter().find(|s| s.name == "Generic").unwrap();
        assert_eq!(sig.auto_add_threshold, None);
    }
}

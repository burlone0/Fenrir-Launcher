use crate::runtime::types::RuntimeType;

/// Parse a runtime directory name into (id, type, version).
/// Returns None if the name doesn't match any known pattern.
pub fn parse_runtime_dir_name(name: &str) -> Option<(&str, RuntimeType, &str)> {
    if let Some(ver) = name.strip_prefix("GE-Proton") {
        Some((name, RuntimeType::ProtonGE, ver))
    } else if let Some(ver) = name.strip_prefix("wine-ge-") {
        Some((name, RuntimeType::WineGE, ver))
    } else if let Some(ver) = name.strip_prefix("Proton ") {
        Some((name, RuntimeType::Proton, ver))
    } else if let Some(ver) = name.strip_prefix("Proton-") {
        Some((name, RuntimeType::Proton, ver))
    } else if name == "wine" || name.starts_with("wine-") {
        let ver = name.strip_prefix("wine-").unwrap_or("system");
        Some((name, RuntimeType::Wine, ver))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proton_ge_version() {
        let v = parse_runtime_dir_name("GE-Proton9-20");
        assert_eq!(v.unwrap(), ("GE-Proton9-20", RuntimeType::ProtonGE, "9-20"));
    }

    #[test]
    fn test_parse_wine_ge_version() {
        let v = parse_runtime_dir_name("wine-ge-8-26");
        assert_eq!(v.unwrap(), ("wine-ge-8-26", RuntimeType::WineGE, "8-26"));
    }

    #[test]
    fn test_parse_proton_valve() {
        let v = parse_runtime_dir_name("Proton 9.0");
        assert_eq!(v.unwrap(), ("Proton 9.0", RuntimeType::Proton, "9.0"));
    }

    #[test]
    fn test_parse_proton_dash_variant() {
        let v = parse_runtime_dir_name("Proton-9.0");
        assert_eq!(v.unwrap(), ("Proton-9.0", RuntimeType::Proton, "9.0"));
    }

    #[test]
    fn test_parse_system_wine() {
        let v = parse_runtime_dir_name("wine");
        assert_eq!(v.unwrap(), ("wine", RuntimeType::Wine, "system"));
    }

    #[test]
    fn test_parse_wine_versioned() {
        let v = parse_runtime_dir_name("wine-9.0");
        assert_eq!(v.unwrap(), ("wine-9.0", RuntimeType::Wine, "9.0"));
    }

    #[test]
    fn test_parse_unknown_returns_none() {
        assert!(parse_runtime_dir_name("random-folder").is_none());
        assert!(parse_runtime_dir_name("").is_none());
        assert!(parse_runtime_dir_name("lutris-runtime").is_none());
    }
}

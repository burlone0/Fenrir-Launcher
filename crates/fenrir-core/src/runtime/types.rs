use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runtime {
    pub id: String,
    pub runtime_type: RuntimeType,
    pub version: String,
    pub path: PathBuf,
    pub source: RuntimeSource,
    pub is_default: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeType {
    Wine,
    Proton,
    ProtonGE,
    WineGE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeSource {
    System,
    Steam,
    Downloaded,
}

impl std::fmt::Display for RuntimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wine => write!(f, "Wine"),
            Self::Proton => write!(f, "Proton"),
            Self::ProtonGE => write!(f, "GE-Proton"),
            Self::WineGE => write!(f, "Wine-GE"),
        }
    }
}

impl std::fmt::Display for RuntimeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "System"),
            Self::Steam => write!(f, "Steam"),
            Self::Downloaded => write!(f, "Downloaded"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_type_display() {
        assert_eq!(RuntimeType::Wine.to_string(), "Wine");
        assert_eq!(RuntimeType::Proton.to_string(), "Proton");
        assert_eq!(RuntimeType::ProtonGE.to_string(), "GE-Proton");
        assert_eq!(RuntimeType::WineGE.to_string(), "Wine-GE");
    }

    #[test]
    fn test_runtime_source_display() {
        assert_eq!(RuntimeSource::System.to_string(), "System");
        assert_eq!(RuntimeSource::Steam.to_string(), "Steam");
        assert_eq!(RuntimeSource::Downloaded.to_string(), "Downloaded");
    }
}

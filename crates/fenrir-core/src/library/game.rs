use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: Uuid,
    pub title: String,
    pub executable: PathBuf,
    pub install_dir: PathBuf,
    pub store_origin: StoreOrigin,
    pub crack_type: Option<CrackType>,
    pub prefix_path: PathBuf,
    pub runtime_id: Option<String>,
    pub status: GameStatus,
    pub play_time: u64,
    pub last_played: Option<DateTime<Utc>>,
    pub added_at: DateTime<Utc>,
    pub user_overrides: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreOrigin {
    Steam,
    GOG,
    Epic,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrackType {
    OnlineFix,
    DODI,
    FitGirl,
    Scene,
    GOGRip,
    SteamRip,
    SmokeAPI,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    Detected,
    Configured,
    Ready,
    Broken,
    NeedsConfirmation,
}

impl std::fmt::Display for StoreOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Steam => write!(f, "Steam"),
            Self::GOG => write!(f, "GOG"),
            Self::Epic => write!(f, "Epic"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for CrackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OnlineFix => write!(f, "OnlineFix"),
            Self::DODI => write!(f, "DODI"),
            Self::FitGirl => write!(f, "FitGirl"),
            Self::Scene => write!(f, "Scene"),
            Self::GOGRip => write!(f, "GOG Rip"),
            Self::SteamRip => write!(f, "Steam Rip"),
            Self::SmokeAPI => write!(f, "SmokeAPI"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Detected => write!(f, "Detected"),
            Self::Configured => write!(f, "Configured"),
            Self::Ready => write!(f, "Ready"),
            Self::Broken => write!(f, "Broken"),
            Self::NeedsConfirmation => write!(f, "NeedsConfirmation"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_origin_display() {
        assert_eq!(StoreOrigin::Steam.to_string(), "Steam");
        assert_eq!(StoreOrigin::GOG.to_string(), "GOG");
        assert_eq!(StoreOrigin::Epic.to_string(), "Epic");
        assert_eq!(StoreOrigin::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_crack_type_display() {
        assert_eq!(CrackType::OnlineFix.to_string(), "OnlineFix");
        assert_eq!(CrackType::DODI.to_string(), "DODI");
        assert_eq!(CrackType::FitGirl.to_string(), "FitGirl");
        assert_eq!(CrackType::Scene.to_string(), "Scene");
        assert_eq!(CrackType::GOGRip.to_string(), "GOG Rip");
        assert_eq!(CrackType::SteamRip.to_string(), "Steam Rip");
        assert_eq!(CrackType::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_game_status_display() {
        assert_eq!(GameStatus::Detected.to_string(), "Detected");
        assert_eq!(GameStatus::Configured.to_string(), "Configured");
        assert_eq!(GameStatus::Ready.to_string(), "Ready");
        assert_eq!(GameStatus::Broken.to_string(), "Broken");
        assert_eq!(
            GameStatus::NeedsConfirmation.to_string(),
            "NeedsConfirmation"
        );
    }
}

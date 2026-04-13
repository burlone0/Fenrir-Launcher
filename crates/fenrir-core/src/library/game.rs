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
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    Detected,
    Configured,
    Ready,
    Broken,
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
        }
    }
}

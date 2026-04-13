use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum FenrirError {
    #[error("config error: {0}")]
    Config(#[from] ConfigError),

    #[error("database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("scanner error: {0}")]
    Scanner(#[from] ScannerError),

    #[error("runtime error: {0}")]
    Runtime(#[from] RuntimeError),

    #[error("prefix error: {0}")]
    Prefix(#[from] PrefixError),

    #[error("launcher error: {0}")]
    Launcher(#[from] LauncherError),
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(PathBuf),

    #[error("failed to parse config: {0}")]
    Parse(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("game not found: {0}")]
    GameNotFound(uuid::Uuid),

    #[error("migration failed: {0}")]
    Migration(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ScannerError {
    #[error("scan directory not found: {0}")]
    DirNotFound(PathBuf),

    #[error("failed to read signatures: {0}")]
    SignatureLoad(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("runtime not found: {0}")]
    NotFound(String),

    #[error("wine/proton not available on system")]
    NoRuntimeAvailable,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PrefixError {
    #[error("wineboot failed: {0}")]
    WinebootFailed(String),

    #[error("prefix directory error: {0}")]
    Directory(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum LauncherError {
    #[error("game not configured: {0}")]
    NotConfigured(uuid::Uuid),

    #[error("executable not found: {0}")]
    ExeNotFound(PathBuf),

    #[error("launch failed: {0}")]
    LaunchFailed(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let e = ConfigError::NotFound(PathBuf::from("/etc/fenrir/config.toml"));
        assert!(e.to_string().contains("config file not found"));

        let e = ConfigError::Parse("unexpected key".to_string());
        assert!(e.to_string().contains("failed to parse config"));
    }

    #[test]
    fn test_database_error_display() {
        let e = DatabaseError::GameNotFound(uuid::Uuid::nil());
        assert!(e.to_string().contains("game not found"));

        let e = DatabaseError::Migration("table missing".to_string());
        assert!(e.to_string().contains("migration failed"));
    }

    #[test]
    fn test_scanner_error_display() {
        let e = ScannerError::DirNotFound(PathBuf::from("/mnt/games"));
        assert!(e.to_string().contains("scan directory not found"));

        let e = ScannerError::SignatureLoad("bad toml".to_string());
        assert!(e.to_string().contains("failed to read signatures"));
    }

    #[test]
    fn test_runtime_error_display() {
        let e = RuntimeError::NotFound("GE-Proton9-20".to_string());
        assert!(e.to_string().contains("runtime not found"));

        let e = RuntimeError::NoRuntimeAvailable;
        assert!(e.to_string().contains("wine/proton not available"));
    }

    #[test]
    fn test_prefix_error_display() {
        let e = PrefixError::WinebootFailed("exit code 1".to_string());
        assert!(e.to_string().contains("wineboot failed"));

        let e = PrefixError::Directory("no such dir".to_string());
        assert!(e.to_string().contains("prefix directory error"));
    }

    #[test]
    fn test_launcher_error_display() {
        let e = LauncherError::NotConfigured(uuid::Uuid::nil());
        assert!(e.to_string().contains("game not configured"));

        let e = LauncherError::ExeNotFound(PathBuf::from("/games/game.exe"));
        assert!(e.to_string().contains("executable not found"));

        let e = LauncherError::LaunchFailed("no such file".to_string());
        assert!(e.to_string().contains("launch failed"));
    }
}

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

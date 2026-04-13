use crate::error::DatabaseError;
use crate::library::game::{CrackType, Game, GameStatus, StoreOrigin};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DatabaseError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self, DatabaseError> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), DatabaseError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS games (
                id              TEXT PRIMARY KEY,
                title           TEXT NOT NULL,
                executable      TEXT NOT NULL,
                install_dir     TEXT NOT NULL,
                store_origin    TEXT NOT NULL,
                crack_type      TEXT,
                prefix_path     TEXT NOT NULL,
                runtime_id      TEXT,
                status          TEXT NOT NULL,
                play_time       INTEGER NOT NULL DEFAULT 0,
                last_played     TEXT,
                added_at        TEXT NOT NULL,
                user_overrides  TEXT
            );

            CREATE TABLE IF NOT EXISTS runtimes (
                id              TEXT PRIMARY KEY,
                runtime_type    TEXT NOT NULL,
                version         TEXT NOT NULL,
                path            TEXT NOT NULL,
                source          TEXT NOT NULL,
                is_default      INTEGER NOT NULL DEFAULT 0
            );",
        )?;
        Ok(())
    }

    pub fn insert_game(&self, game: &Game) -> Result<(), DatabaseError> {
        self.conn.execute(
            "INSERT INTO games (id, title, executable, install_dir, store_origin,
             crack_type, prefix_path, runtime_id, status, play_time, last_played,
             added_at, user_overrides)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                game.id.to_string(),
                game.title,
                game.executable.to_string_lossy().to_string(),
                game.install_dir.to_string_lossy().to_string(),
                format!("{}", game.store_origin),
                game.crack_type.map(|c| format!("{}", c)),
                game.prefix_path.to_string_lossy().to_string(),
                game.runtime_id,
                format!("{}", game.status),
                game.play_time as i64,
                game.last_played.map(|d| d.to_rfc3339()),
                game.added_at.to_rfc3339(),
                game.user_overrides.as_ref().map(|v| v.to_string()),
            ],
        )?;
        Ok(())
    }

    pub fn get_game(&self, id: Uuid) -> Result<Option<Game>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, executable, install_dir, store_origin, crack_type,
             prefix_path, runtime_id, status, play_time, last_played, added_at,
             user_overrides FROM games WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        match rows.next()? {
            Some(row) => Ok(Some(row_to_game(row)?)),
            None => Ok(None),
        }
    }

    pub fn list_games(&self) -> Result<Vec<Game>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, executable, install_dir, store_origin, crack_type,
             prefix_path, runtime_id, status, play_time, last_played, added_at,
             user_overrides FROM games ORDER BY title",
        )?;

        let games = stmt
            .query_map([], |row| {
                row_to_game(row).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(games)
    }

    pub fn update_game(&self, game: &Game) -> Result<(), DatabaseError> {
        let affected = self.conn.execute(
            "UPDATE games SET title=?2, executable=?3, install_dir=?4,
             store_origin=?5, crack_type=?6, prefix_path=?7, runtime_id=?8,
             status=?9, play_time=?10, last_played=?11, user_overrides=?12
             WHERE id=?1",
            params![
                game.id.to_string(),
                game.title,
                game.executable.to_string_lossy().to_string(),
                game.install_dir.to_string_lossy().to_string(),
                format!("{}", game.store_origin),
                game.crack_type.map(|c| format!("{}", c)),
                game.prefix_path.to_string_lossy().to_string(),
                game.runtime_id,
                format!("{}", game.status),
                game.play_time as i64,
                game.last_played.map(|d| d.to_rfc3339()),
                game.user_overrides.as_ref().map(|v| v.to_string()),
            ],
        )?;
        if affected == 0 {
            return Err(DatabaseError::GameNotFound(game.id));
        }
        Ok(())
    }

    pub fn delete_game(&self, id: Uuid) -> Result<(), DatabaseError> {
        self.conn
            .execute("DELETE FROM games WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    pub fn find_by_title(&self, query: &str) -> Result<Vec<Game>, DatabaseError> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, title, executable, install_dir, store_origin, crack_type,
             prefix_path, runtime_id, status, play_time, last_played, added_at,
             user_overrides FROM games WHERE title LIKE ?1 COLLATE NOCASE ORDER BY title",
        )?;

        let games = stmt
            .query_map(params![pattern], |row| {
                row_to_game(row).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(games)
    }
}

fn row_to_game(row: &rusqlite::Row) -> Result<Game, DatabaseError> {
    let id_str: String = row.get(0)?;
    let store_str: String = row.get(4)?;
    let crack_str: Option<String> = row.get(5)?;
    let status_str: String = row.get(8)?;
    let last_played_str: Option<String> = row.get(10)?;
    let added_str: String = row.get(11)?;
    let overrides_str: Option<String> = row.get(12)?;

    Ok(Game {
        id: Uuid::parse_str(&id_str).map_err(|e| DatabaseError::Migration(e.to_string()))?,
        title: row.get(1)?,
        executable: PathBuf::from(row.get::<_, String>(2)?),
        install_dir: PathBuf::from(row.get::<_, String>(3)?),
        store_origin: parse_store_origin(&store_str),
        crack_type: crack_str.as_deref().map(parse_crack_type),
        prefix_path: PathBuf::from(row.get::<_, String>(6)?),
        runtime_id: row.get(7)?,
        status: parse_game_status(&status_str),
        play_time: row.get::<_, i64>(9)? as u64,
        last_played: last_played_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|d| d.with_timezone(&Utc))
        }),
        added_at: DateTime::parse_from_rfc3339(&added_str)
            .map(|d| d.with_timezone(&Utc))
            .map_err(|e| DatabaseError::Migration(e.to_string()))?,
        user_overrides: overrides_str.and_then(|s| serde_json::from_str(&s).ok()),
    })
}

fn parse_store_origin(s: &str) -> StoreOrigin {
    match s {
        "Steam" => StoreOrigin::Steam,
        "GOG" => StoreOrigin::GOG,
        "Epic" => StoreOrigin::Epic,
        _ => StoreOrigin::Unknown,
    }
}

fn parse_crack_type(s: &str) -> CrackType {
    match s {
        "OnlineFix" => CrackType::OnlineFix,
        "DODI" => CrackType::DODI,
        "FitGirl" => CrackType::FitGirl,
        "Scene" => CrackType::Scene,
        "GOG Rip" => CrackType::GOGRip,
        _ => CrackType::Unknown,
    }
}

fn parse_game_status(s: &str) -> GameStatus {
    match s {
        "Detected" => GameStatus::Detected,
        "Configured" => GameStatus::Configured,
        "Ready" => GameStatus::Ready,
        _ => GameStatus::Broken,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_game(title: &str) -> Game {
        Game {
            id: Uuid::new_v4(),
            title: title.to_string(),
            executable: PathBuf::from("/games/test/game.exe"),
            install_dir: PathBuf::from("/games/test"),
            store_origin: StoreOrigin::Steam,
            crack_type: Some(CrackType::Unknown),
            prefix_path: PathBuf::from("/tmp/prefix"),
            runtime_id: None,
            status: GameStatus::Detected,
            play_time: 0,
            last_played: None,
            added_at: Utc::now(),
            user_overrides: None,
        }
    }

    #[test]
    fn test_open_in_memory() {
        let db = Database::open_in_memory().unwrap();
        assert!(db.conn.is_autocommit());
    }

    #[test]
    fn test_migration_creates_tables() {
        let db = Database::open_in_memory().unwrap();
        let count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='games'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_and_get_game() {
        let db = Database::open_in_memory().unwrap();
        let game = make_test_game("Test Game");
        let id = game.id;

        db.insert_game(&game).unwrap();
        let fetched = db.get_game(id).unwrap().unwrap();
        assert_eq!(fetched.title, "Test Game");
        assert_eq!(fetched.store_origin, StoreOrigin::Steam);
    }

    #[test]
    fn test_list_games() {
        let db = Database::open_in_memory().unwrap();
        assert_eq!(db.list_games().unwrap().len(), 0);

        db.insert_game(&make_test_game("Game 1")).unwrap();
        assert_eq!(db.list_games().unwrap().len(), 1);
    }

    #[test]
    fn test_update_game() {
        let db = Database::open_in_memory().unwrap();
        let mut game = make_test_game("Original");
        db.insert_game(&game).unwrap();

        game.title = "Updated".to_string();
        game.status = GameStatus::Configured;
        db.update_game(&game).unwrap();

        let fetched = db.get_game(game.id).unwrap().unwrap();
        assert_eq!(fetched.title, "Updated");
        assert_eq!(fetched.status, GameStatus::Configured);
    }

    #[test]
    fn test_delete_game() {
        let db = Database::open_in_memory().unwrap();
        let game = make_test_game("To Delete");
        let id = game.id;

        db.insert_game(&game).unwrap();
        db.delete_game(id).unwrap();
        assert!(db.get_game(id).unwrap().is_none());
    }

    #[test]
    fn test_find_by_title_fuzzy() {
        let db = Database::open_in_memory().unwrap();
        db.insert_game(&make_test_game("Elden Ring")).unwrap();
        db.insert_game(&make_test_game("Dark Souls III")).unwrap();

        let results = db.find_by_title("elden").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Elden Ring");
    }
}

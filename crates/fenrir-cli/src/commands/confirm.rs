use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::library::db::Database;
use fenrir_core::library::game::GameStatus;

pub fn run(query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = FenrirConfig::load()?;
    let db = Database::open(&config.general.library_db)?;
    confirm_game(&db, query)
}

fn confirm_game(db: &Database, query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let game = if let Ok(uuid) = uuid::Uuid::parse_str(query) {
        db.get_game(uuid)?
    } else {
        db.find_by_title(query)?
            .into_iter()
            .find(|g| g.status == GameStatus::NeedsConfirmation)
    };

    match game {
        Some(g) if g.status == GameStatus::NeedsConfirmation => {
            let mut confirmed = g.clone();
            confirmed.status = GameStatus::Detected;
            db.update_game(&confirmed)?;
            println!("confirmed: {} is now Detected", confirmed.title);
            Ok(())
        }
        Some(g) => {
            eprintln!(
                "game '{}' has status '{}', not NeedsConfirmation",
                g.title, g.status
            );
            Err(format!("game '{}' is not pending confirmation", g.title).into())
        }
        None => {
            eprintln!("no game pending confirmation found for: {}", query);
            Err(format!("game not found: {}", query).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use fenrir_core::library::db::Database;
    use fenrir_core::library::game::{CrackType, Game, GameStatus, StoreOrigin};
    use std::path::PathBuf;
    use uuid::Uuid;

    fn make_game(title: &str, status: GameStatus) -> Game {
        Game {
            id: Uuid::new_v4(),
            title: title.to_string(),
            executable: PathBuf::from("/games/game.exe"),
            install_dir: PathBuf::from("/games"),
            store_origin: StoreOrigin::Steam,
            crack_type: Some(CrackType::Unknown),
            prefix_path: PathBuf::new(),
            runtime_id: None,
            status,
            play_time: 0,
            last_played: None,
            added_at: Utc::now(),
            user_overrides: None,
        }
    }

    #[test]
    fn test_confirm_by_title_promotes_to_detected() {
        let db = Database::open_in_memory().unwrap();
        let game = make_game("Half-Life 3", GameStatus::NeedsConfirmation);
        let id = game.id;
        db.insert_game(&game).unwrap();

        confirm_game(&db, "Half-Life 3").unwrap();

        let fetched = db.get_game(id).unwrap().unwrap();
        assert_eq!(fetched.status, GameStatus::Detected);
    }

    #[test]
    fn test_confirm_by_uuid_promotes_to_detected() {
        let db = Database::open_in_memory().unwrap();
        let game = make_game("Portal 3", GameStatus::NeedsConfirmation);
        let id = game.id;
        db.insert_game(&game).unwrap();

        confirm_game(&db, &id.to_string()).unwrap();

        let fetched = db.get_game(id).unwrap().unwrap();
        assert_eq!(fetched.status, GameStatus::Detected);
    }

    #[test]
    fn test_confirm_not_found_returns_error() {
        let db = Database::open_in_memory().unwrap();
        let result = confirm_game(&db, "nonexistent game xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_confirm_wrong_status_returns_error() {
        let db = Database::open_in_memory().unwrap();
        let game = make_game("Already Detected", GameStatus::Detected);
        db.insert_game(&game).unwrap();

        let result = confirm_game(&db, "Already Detected");
        assert!(result.is_err());
    }

    #[test]
    fn test_confirm_by_title_partial_match_needs_confirmation_status() {
        let db = Database::open_in_memory().unwrap();
        db.insert_game(&make_game("Cyberpunk 2077", GameStatus::Detected))
            .unwrap();
        let pending = make_game("Cyberpunk 2099", GameStatus::NeedsConfirmation);
        let id = pending.id;
        db.insert_game(&pending).unwrap();

        confirm_game(&db, "Cyberpunk 2099").unwrap();

        let fetched = db.get_game(id).unwrap().unwrap();
        assert_eq!(fetched.status, GameStatus::Detected);
    }
}

use crate::library::game::{CrackType, StoreOrigin};
use crate::scanner::detector::GameCandidate;
use crate::scanner::signatures::Signature;
use std::path::Path;
use tracing::debug;

const SCORE_REQUIRED: u32 = 30;
const SCORE_OPTIONAL: u32 = 15;
const SCORE_BOOST: u32 = 10;
const THRESHOLD_HIGH: u32 = 60;
const THRESHOLD_LOW: u32 = 30;

#[derive(Debug, Clone)]
pub struct ClassifiedGame {
    pub path: std::path::PathBuf,
    pub exe_files: Vec<std::path::PathBuf>,
    pub title: String,
    pub store_origin: StoreOrigin,
    pub crack_type: Option<CrackType>,
    pub confidence: u32,
    pub signature_name: String,
    pub high_confidence_threshold: u32,
}

pub fn classify_candidate(
    candidate: &GameCandidate,
    signatures: &[Signature],
) -> Option<(u32, ClassifiedGame)> {
    let mut best_score = 0u32;
    let mut best_match: Option<ClassifiedGame> = None;

    for sig in signatures {
        let score = score_candidate(candidate, sig);
        if score > best_score && score >= THRESHOLD_LOW {
            best_score = score;
            best_match = Some(ClassifiedGame {
                path: candidate.path.clone(),
                exe_files: candidate.exe_files.clone(),
                title: extract_title(&candidate.path),
                store_origin: parse_store(&sig.store),
                crack_type: sig.crack_type.as_deref().map(parse_crack),
                confidence: score,
                signature_name: sig.name.clone(),
                high_confidence_threshold: sig
                    .auto_add_threshold
                    .unwrap_or(THRESHOLD_HIGH)
                    .max(THRESHOLD_LOW),
            });
        }
    }

    best_match.map(|m| (best_score, m))
}

fn score_candidate(candidate: &GameCandidate, sig: &Signature) -> u32 {
    let all_required = sig
        .required_files
        .iter()
        .all(|f| file_exists_in_dir(&candidate.path, f));
    if !all_required {
        return 0;
    }

    let mut score = sig.required_files.len() as u32 * SCORE_REQUIRED;

    for f in &sig.optional_files {
        if file_exists_in_dir(&candidate.path, f) {
            score += SCORE_OPTIONAL;
        }
    }

    for f in &sig.confidence_boost {
        if file_exists_in_dir(&candidate.path, f) {
            score += SCORE_BOOST;
        }
    }

    debug!(
        "score for {} against '{}': {}",
        candidate.path.display(),
        sig.name,
        score
    );
    score
}

fn file_exists_in_dir(dir: &Path, pattern: &str) -> bool {
    if pattern.ends_with('/') {
        let dir_name = pattern.trim_end_matches('/');
        dir.join(dir_name).is_dir()
    } else if pattern.contains('*') {
        let glob_pattern = format!("{}/{}", dir.display(), pattern);
        glob::glob(&glob_pattern)
            .map(|paths| paths.filter_map(|p| p.ok()).next().is_some())
            .unwrap_or(false)
    } else {
        // Exact match, then case-insensitive fallback
        if dir.join(pattern).exists() {
            return true;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.eq_ignore_ascii_case(pattern) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

fn extract_title(path: &Path) -> String {
    let dirname = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown");
    clean_title(dirname)
}

pub fn clean_title(name: &str) -> String {
    let mut title = name.to_string();

    // Remove [tags]
    while let (Some(start), Some(end)) = (title.find('['), title.find(']')) {
        if start < end {
            title = format!("{}{}", &title[..start], &title[end + 1..]);
        } else {
            break;
        }
    }

    // Remove (tags)
    while let (Some(start), Some(end)) = (title.find('('), title.find(')')) {
        if start < end {
            title = format!("{}{}", &title[..start], &title[end + 1..]);
        } else {
            break;
        }
    }

    // Remove trailing version patterns BEFORE dot replacement (e.g. .v2.1.3)
    let re_ver = regex_lite::Regex::new(r"[.\s-]*v?\d+\.\d+(\.\d+)*\s*$").unwrap();
    title = re_ver.replace(&title, "").to_string();

    // Dots to spaces
    title = title.replace('.', " ");

    title.trim().to_string()
}

fn parse_store(store: &Option<String>) -> StoreOrigin {
    match store.as_deref() {
        Some("Steam") => StoreOrigin::Steam,
        Some("GOG") => StoreOrigin::GOG,
        Some("Epic") => StoreOrigin::Epic,
        _ => StoreOrigin::Unknown,
    }
}

fn parse_crack(s: &str) -> CrackType {
    match s {
        "OnlineFix" => CrackType::OnlineFix,
        "DODI" => CrackType::DODI,
        "FitGirl" => CrackType::FitGirl,
        "Scene" => CrackType::Scene,
        "GOGRip" => CrackType::GOGRip,
        "SteamRIP" => CrackType::SteamRip,
        _ => CrackType::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn steam_signature() -> Signature {
        Signature {
            name: "Steam Generic".to_string(),
            store: Some("Steam".to_string()),
            crack_type: None,
            required_files: vec!["steam_api.dll".to_string()],
            optional_files: vec!["steam_api64.dll".to_string(), "steam_appid.txt".to_string()],
            confidence_boost: vec!["steam_emu.ini".to_string()],
            auto_add_threshold: None,
        }
    }

    fn onlinefix_signature() -> Signature {
        Signature {
            name: "OnlineFix".to_string(),
            store: Some("Steam".to_string()),
            crack_type: Some("OnlineFix".to_string()),
            required_files: vec!["OnlineFix.url".to_string()],
            optional_files: vec!["OnlineFix64.dll".to_string()],
            confidence_boost: vec![],
            auto_add_threshold: None,
        }
    }

    #[test]
    fn test_high_confidence_match() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("steam_api.dll"), "fake").unwrap();
        fs::write(dir.path().join("steam_api64.dll"), "fake").unwrap();
        fs::write(dir.path().join("steam_appid.txt"), "12345").unwrap();
        fs::write(dir.path().join("game.exe"), "fake").unwrap();

        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![dir.path().join("game.exe")],
        };

        let result = classify_candidate(&candidate, &[steam_signature()]);
        assert!(result.is_some());
        let (score, _) = result.unwrap();
        // required(30) + optional(15+15) = 60
        assert!(score >= THRESHOLD_HIGH);
    }

    #[test]
    fn test_low_confidence_match() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("steam_api.dll"), "fake").unwrap();
        fs::write(dir.path().join("game.exe"), "fake").unwrap();

        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![dir.path().join("game.exe")],
        };

        let result = classify_candidate(&candidate, &[steam_signature()]);
        assert!(result.is_some());
        let (score, _) = result.unwrap();
        // required(30) only — between thresholds
        assert!((THRESHOLD_LOW..THRESHOLD_HIGH).contains(&score));
    }

    #[test]
    fn test_no_match() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("game.exe"), "fake").unwrap();

        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![dir.path().join("game.exe")],
        };

        let result = classify_candidate(&candidate, &[steam_signature()]);
        assert!(result.is_none());
    }

    #[test]
    fn test_best_signature_wins() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("steam_api.dll"), "fake").unwrap();
        fs::write(dir.path().join("OnlineFix.url"), "fake").unwrap();
        fs::write(dir.path().join("OnlineFix64.dll"), "fake").unwrap();
        fs::write(dir.path().join("game.exe"), "fake").unwrap();

        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![dir.path().join("game.exe")],
        };

        let sigs = vec![steam_signature(), onlinefix_signature()];
        let result = classify_candidate(&candidate, &sigs);
        assert!(result.is_some());
        let (_, classified) = result.unwrap();
        // OnlineFix has higher score: required(30) + optional(15) = 45
        // vs Steam: required(30) = 30
        assert_eq!(classified.crack_type, Some(CrackType::OnlineFix));
    }

    #[test]
    fn test_clean_title_brackets() {
        assert_eq!(clean_title("Elden Ring [FitGirl Repack]"), "Elden Ring");
    }

    #[test]
    fn test_clean_title_dots_and_version() {
        assert_eq!(clean_title("Cyberpunk.2077.v2.1"), "Cyberpunk 2077");
    }

    #[test]
    fn test_clean_title_parentheses() {
        assert_eq!(clean_title("Dark Souls III (GOG)"), "Dark Souls III");
    }

    fn onlinefix_with_low_threshold() -> Signature {
        Signature {
            name: "OnlineFix Low Threshold".to_string(),
            store: Some("Steam".to_string()),
            crack_type: Some("OnlineFix".to_string()),
            required_files: vec!["OnlineFix.ini".to_string()],
            optional_files: vec![],
            confidence_boost: vec![],
            auto_add_threshold: Some(30),
        }
    }

    #[test]
    fn test_auto_add_threshold_promotes_to_high() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("OnlineFix.ini"), "fake").unwrap();
        fs::write(dir.path().join("game.exe"), "fake").unwrap();

        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![dir.path().join("game.exe")],
        };

        let sig = onlinefix_with_low_threshold();
        let result = classify_candidate(&candidate, &[sig]).unwrap();
        let (score, classified) = result;
        // score = 30 (1 required), threshold = 30 → high confidence
        assert_eq!(score, 30);
        assert_eq!(classified.high_confidence_threshold, 30);
    }

    #[test]
    fn test_no_auto_add_threshold_uses_default_60() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("OnlineFix.ini"), "fake").unwrap();
        fs::write(dir.path().join("game.exe"), "fake").unwrap();

        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![dir.path().join("game.exe")],
        };

        // Same sig but WITHOUT auto_add_threshold
        let sig = Signature {
            name: "OnlineFix No Threshold".to_string(),
            store: Some("Steam".to_string()),
            crack_type: Some("OnlineFix".to_string()),
            required_files: vec!["OnlineFix.ini".to_string()],
            optional_files: vec![],
            confidence_boost: vec![],
            auto_add_threshold: None,
        };
        let result = classify_candidate(&candidate, &[sig]).unwrap();
        let (score, classified) = result;
        assert_eq!(score, 30);
        // high_confidence_threshold = 60 (default) → score < threshold → needs confirmation
        assert_eq!(classified.high_confidence_threshold, 60);
    }
}

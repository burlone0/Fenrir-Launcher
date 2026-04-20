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
                title: extract_title(&compute_best_title_path(
                    &candidate.path,
                    &candidate.exe_files,
                )),
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

    // Dirname fallback for DRM-free SteamRIP releases that have no steam_api*.dll or marker
    // files. SteamRIP always names the root folder with a "-SteamRIP.com" or "-SteamRIP"
    // suffix, which is a reliable enough identifier to auto-add.
    if best_score == 0 {
        if let Some(name) = candidate.path.file_name().and_then(|n| n.to_str()) {
            let upper = name.to_uppercase();
            if upper.ends_with("-STEAMRIP.COM") || upper.ends_with("-STEAMRIP") {
                let title = extract_title(&candidate.path);
                best_score = THRESHOLD_LOW;
                best_match = Some(ClassifiedGame {
                    path: candidate.path.clone(),
                    exe_files: candidate.exe_files.clone(),
                    title,
                    store_origin: StoreOrigin::Steam,
                    crack_type: Some(CrackType::SteamRip),
                    confidence: THRESHOLD_LOW,
                    signature_name: "SteamRIP (dirname)".to_string(),
                    high_confidence_threshold: THRESHOLD_LOW,
                });
            }
        }
    }

    best_match.map(|m| (best_score, m))
}

fn score_candidate(candidate: &GameCandidate, sig: &Signature) -> u32 {
    let missing: Vec<&str> = sig
        .required_files
        .iter()
        .filter(|f| !file_exists_in_dir(&candidate.path, f))
        .map(|f| f.as_str())
        .collect();
    if !missing.is_empty() {
        debug!(
            "skip '{}' for {}: missing required [{}]",
            sig.name,
            candidate.path.display(),
            missing.join(", ")
        );
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
        return dir.join(dir_name).is_dir();
    }

    if pattern.contains('*') {
        let glob_pattern = format!("{}/{}", dir.display(), pattern);
        return glob::glob(&glob_pattern)
            .map(|paths| paths.filter_map(|p| p.ok()).next().is_some())
            .unwrap_or(false);
    }

    // Exact match, then case-insensitive fallback at root
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

    // UE deep scan: crack files live in Binaries/Win64/ (or Win32/) — either directly
    // inside the candidate dir, or one subdir level deeper (launcher-stub layout).
    // Covers: DontScream-style (dir/Binaries/Win64/marker) and
    //         KEEP GAMBLING-style (dir/GameName/Binaries/Win64/marker).
    const UE_BIN_MARKERS: &[&str] = &[
        "onlinefix.ini",
        "onlinefix64.dll",
        "onlinefix.url",
        "unsteam.ini",
        "unsteam.dll",
        "steam_api.dll",
        "steam_api64.dll",
        "steam_appid.txt",
    ];
    let pattern_lower = pattern.to_lowercase();
    if UE_BIN_MARKERS.contains(&pattern_lower.as_str()) {
        for bin_subdir in &["Binaries/Win64", "Binaries/Win32"] {
            // Direct: <dir>/Binaries/Win64/<file>
            let bin_dir = dir.join(bin_subdir);
            if bin_dir.join(pattern).exists() {
                return true;
            }
            if let Ok(bin_entries) = std::fs::read_dir(&bin_dir) {
                for be in bin_entries.flatten() {
                    if let Some(name) = be.file_name().to_str() {
                        if name.eq_ignore_ascii_case(pattern) {
                            return true;
                        }
                    }
                }
            }
        }
        // One subdir deeper: <dir>/<name>/Binaries/Win64/<file>
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let sub = entry.path();
                if !sub.is_dir() {
                    continue;
                }
                for bin_subdir in &["Binaries/Win64", "Binaries/Win32"] {
                    let bin_dir = sub.join(bin_subdir);
                    if bin_dir.join(pattern).exists() {
                        return true;
                    }
                    if let Ok(bin_entries) = std::fs::read_dir(&bin_dir) {
                        for be in bin_entries.flatten() {
                            if let Some(name) = be.file_name().to_str() {
                                if name.eq_ignore_ascii_case(pattern) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Unity deep scan: check <subdir>/Plugins/x86_64/ and <subdir>/Plugins/x86/
    // Only for steam_api*.dll — other signatures don't hide their files in plugin dirs.
    if pattern_lower == "steam_api.dll" || pattern_lower == "steam_api64.dll" {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let sub = entry.path();
                if !sub.is_dir() {
                    continue;
                }
                for plugin_subdir in &["Plugins/x86_64", "Plugins/x86"] {
                    let plugin_dir = sub.join(plugin_subdir);
                    if plugin_dir.join(pattern).exists() {
                        return true;
                    }
                    if let Ok(plugin_entries) = std::fs::read_dir(&plugin_dir) {
                        for pe in plugin_entries.flatten() {
                            if let Some(name) = pe.file_name().to_str() {
                                if name.eq_ignore_ascii_case(pattern) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

fn extract_title(path: &Path) -> String {
    let dirname = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown");
    clean_title(dirname)
}

/// Returns true if `dir` is a distribution wrapper (not the real game directory).
/// Detection via URL marker file (preferred) or AstralGames naming convention (~AG suffix).
fn is_wrapper_dir(dir: &Path) -> bool {
    const WRAPPER_MARKERS: &[&str] = &["AstralGames ~ Pre-Installed Games.url"];
    for marker in WRAPPER_MARKERS {
        if dir.join(marker).exists() {
            return true;
        }
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                for marker in WRAPPER_MARKERS {
                    if name.eq_ignore_ascii_case(marker) {
                        return true;
                    }
                }
            }
        }
    }
    // AstralGames uploaders name the outer folder with a "~AG" suffix even when the URL
    // shortcut was deleted. Match case-insensitively so "~ag" variants are also caught.
    if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
        if name.to_uppercase().ends_with("~AG") {
            return true;
        }
    }
    false
}

/// Returns the direct child directory of `game_root` that is an ancestor of `exe`.
/// Returns `None` when `exe` is directly inside `game_root` (no subfolder between them).
fn direct_child_toward_exe(game_root: &Path, exe: &Path) -> Option<std::path::PathBuf> {
    let mut path = exe;
    loop {
        let parent = path.parent()?;
        if parent == game_root {
            return if path.is_dir() {
                Some(path.to_path_buf())
            } else {
                None
            };
        }
        path = parent;
    }
}

/// Picks the best path to extract the game title from.
/// For wrapper directories (e.g. AstralGames), uses the subfolder that contains the exe.
/// Iterates all exe files because the first entry may be a root-level launcher/setup that
/// would resolve to None, while the real game exe is in a subfolder.
fn compute_best_title_path(
    game_root: &Path,
    exe_files: &[std::path::PathBuf],
) -> std::path::PathBuf {
    if is_wrapper_dir(game_root) {
        for exe in exe_files {
            if let Some(child) = direct_child_toward_exe(game_root, exe) {
                return child;
            }
        }
    }
    game_root.to_path_buf()
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

    // Strip distribution-site suffixes before dot replacement (e.g. "Game-SteamRIP.com" → "Game")
    let re_site = regex_lite::Regex::new(r"(?i)[-.]steamrip(?:\.com)?$").unwrap();
    title = re_site.replace(&title, "").to_string();

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
            cleanup_files: vec![],
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
            cleanup_files: vec![],
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
            cleanup_files: vec![],
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
            cleanup_files: vec![],
        };
        let result = classify_candidate(&candidate, &[sig]).unwrap();
        let (score, classified) = result;
        assert_eq!(score, 30);
        // high_confidence_threshold = 60 (default) → score < threshold → needs confirmation
        assert_eq!(classified.high_confidence_threshold, 60);
    }

    #[test]
    fn test_unity_deep_scan_finds_steam_api64_in_plugins_x86_64() {
        let dir = tempfile::tempdir().unwrap();
        let plugins = dir
            .path()
            .join("GameName_Data")
            .join("Plugins")
            .join("x86_64");
        fs::create_dir_all(&plugins).unwrap();
        fs::write(plugins.join("steam_api64.dll"), "fake").unwrap();
        fs::write(dir.path().join("game.exe"), "fake").unwrap();

        // file_exists_in_dir is private — test via score_candidate
        let sig = Signature {
            name: "Test".to_string(),
            store: Some("Steam".to_string()),
            crack_type: None,
            required_files: vec!["steam_api64.dll".to_string()],
            optional_files: vec![],
            confidence_boost: vec![],
            auto_add_threshold: None,
            cleanup_files: vec![],
        };
        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![dir.path().join("game.exe")],
        };
        let result = classify_candidate(&candidate, &[sig]);
        assert!(
            result.is_some(),
            "steam_api64.dll in Plugins/x86_64 must be detected"
        );
    }

    #[test]
    fn test_unity_deep_scan_finds_steam_api_in_plugins_x86() {
        let dir = tempfile::tempdir().unwrap();
        let plugins = dir.path().join("SomeGame_Data").join("Plugins").join("x86");
        fs::create_dir_all(&plugins).unwrap();
        fs::write(plugins.join("steam_api.dll"), "fake").unwrap();
        fs::write(dir.path().join("game.exe"), "fake").unwrap();

        let sig = Signature {
            name: "Test".to_string(),
            store: Some("Steam".to_string()),
            crack_type: None,
            required_files: vec!["steam_api.dll".to_string()],
            optional_files: vec![],
            confidence_boost: vec![],
            auto_add_threshold: None,
            cleanup_files: vec![],
        };
        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![dir.path().join("game.exe")],
        };
        let result = classify_candidate(&candidate, &[sig]);
        assert!(
            result.is_some(),
            "steam_api.dll in Plugins/x86 must be detected"
        );
    }

    #[test]
    fn test_steamrip_dirname_fallback_drm_free() {
        // DRM-free SteamRIP game: no steam_api*.dll, no marker files inside the folder.
        // Detection must fall back to the directory name convention.
        let base = tempfile::tempdir().unwrap();
        let game_dir = base
            .path()
            .join("Keep-Talking-and-Nobody-Explodes-SteamRIP.com");
        fs::create_dir(&game_dir).unwrap();
        fs::write(game_dir.join("KeepTalking.exe"), "fake").unwrap();

        let candidate = GameCandidate {
            path: game_dir.clone(),
            exe_files: vec![game_dir.join("KeepTalking.exe")],
        };
        let result = classify_candidate(&candidate, &[steam_signature()]);
        assert!(
            result.is_some(),
            "DRM-free SteamRIP game must be detected via dirname"
        );
        let (score, classified) = result.unwrap();
        assert_eq!(score, THRESHOLD_LOW);
        assert_eq!(classified.high_confidence_threshold, THRESHOLD_LOW);
        assert_eq!(classified.crack_type, Some(CrackType::SteamRip));
        assert_eq!(classified.title, "Keep-Talking-and-Nobody-Explodes");
    }

    #[test]
    fn test_clean_title_strips_steamrip_suffix() {
        assert_eq!(clean_title("Egging-On-SteamRIP.com"), "Egging-On");
        assert_eq!(clean_title("Keep Talking-SteamRIP"), "Keep Talking");
        assert_eq!(clean_title("Papers Please.SteamRIP"), "Papers Please");
    }

    #[test]
    fn test_classify_uses_subfolder_title_for_astral_games_wrapper() {
        // Real layout: wrapper root has AstralGames URL, game files live one level deeper.
        // steam_api64.dll is in YAPYAP/Plugins/x86_64/ (Unity layout) so the Unity deep scan
        // finds it when scoring from the wrapper root, while the exe is in YAPYAP/.
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("AstralGames ~ Pre-Installed Games.url"),
            "fake",
        )
        .unwrap();
        let game_sub = dir.path().join("YAPYAP");
        let plugins = game_sub.join("Plugins").join("x86_64");
        fs::create_dir_all(&plugins).unwrap();
        fs::write(plugins.join("steam_api64.dll"), "fake").unwrap();
        fs::write(game_sub.join("yapyap.exe"), "fake").unwrap();

        let sig = Signature {
            name: "Steam Generic 64".to_string(),
            store: Some("Steam".to_string()),
            crack_type: None,
            required_files: vec!["steam_api64.dll".to_string()],
            optional_files: vec![],
            confidence_boost: vec![],
            auto_add_threshold: None,
            cleanup_files: vec![],
        };

        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![game_sub.join("yapyap.exe")],
        };
        let result = classify_candidate(&candidate, &[sig]);
        assert!(
            result.is_some(),
            "must detect steam_api64.dll via Unity deep scan through wrapper"
        );
        let (_, classified) = result.unwrap();
        assert_eq!(
            classified.title, "YAPYAP",
            "title must come from subfolder, not wrapper root"
        );
    }

    #[test]
    fn test_classify_uses_subfolder_title_for_ag_named_wrapper_no_url_file() {
        // URL file deleted: wrapper detected by "~AG" folder name suffix alone.
        let base = tempfile::tempdir().unwrap();
        let wrapper = base.path().join("YY B21832759~AG");
        let game_sub = wrapper.join("YAPYAP");
        let plugins = game_sub.join("Plugins").join("x86_64");
        fs::create_dir_all(&plugins).unwrap();
        fs::write(plugins.join("steam_api64.dll"), "fake").unwrap();
        fs::write(game_sub.join("yapyap.exe"), "fake").unwrap();

        let sig = Signature {
            name: "Steam Generic 64".to_string(),
            store: Some("Steam".to_string()),
            crack_type: None,
            required_files: vec!["steam_api64.dll".to_string()],
            optional_files: vec![],
            confidence_boost: vec![],
            auto_add_threshold: None,
            cleanup_files: vec![],
        };
        let candidate = GameCandidate {
            path: wrapper.clone(),
            exe_files: vec![game_sub.join("yapyap.exe")],
        };
        let result = classify_candidate(&candidate, &[sig]);
        assert!(result.is_some(), "must detect via Unity deep scan");
        let (_, classified) = result.unwrap();
        assert_eq!(
            classified.title, "YAPYAP",
            "~AG name must trigger wrapper detection"
        );
    }

    #[test]
    fn test_unity_deep_scan_not_triggered_for_non_steam_api_files() {
        let dir = tempfile::tempdir().unwrap();
        // Put OnlineFix.ini only in a deep subdirectory — should NOT be found
        let subdir = dir.path().join("SomeData").join("Plugins").join("x86_64");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("OnlineFix.ini"), "fake").unwrap();
        fs::write(dir.path().join("game.exe"), "fake").unwrap();

        let sig = Signature {
            name: "Test".to_string(),
            store: Some("Steam".to_string()),
            crack_type: Some("OnlineFix".to_string()),
            required_files: vec!["OnlineFix.ini".to_string()],
            optional_files: vec![],
            confidence_boost: vec![],
            auto_add_threshold: None,
            cleanup_files: vec![],
        };
        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![dir.path().join("game.exe")],
        };
        let result = classify_candidate(&candidate, &[sig]);
        assert!(
            result.is_none(),
            "non-steam-api files must NOT be found in deep scan"
        );
    }

    // -----------------------------------------------------------------------
    // UE deep scan: markers buried in Binaries/Win64/ or <subdir>/Binaries/Win64/
    // -----------------------------------------------------------------------

    fn unsteam_signature() -> Signature {
        Signature {
            name: "unsteam".to_string(),
            store: Some("Steam".to_string()),
            crack_type: None,
            required_files: vec!["unsteam.ini".to_string()],
            optional_files: vec!["unsteam.dll".to_string()],
            confidence_boost: vec![],
            auto_add_threshold: Some(30),
            cleanup_files: vec![],
        }
    }

    fn onlinefix_ue_signature() -> Signature {
        Signature {
            name: "OnlineFix UE".to_string(),
            store: Some("Steam".to_string()),
            crack_type: Some("OnlineFix".to_string()),
            required_files: vec!["OnlineFix.ini".to_string()],
            optional_files: vec![],
            confidence_boost: vec![],
            auto_add_threshold: Some(30),
            cleanup_files: vec![],
        }
    }

    #[test]
    fn test_ue_deep_scan_finds_unsteam_ini_in_binaries_win64_direct() {
        // Mimics DontScream: candidate root has no marker at root level,
        // but unsteam.ini lives directly in Binaries/Win64/.
        let dir = tempfile::tempdir().unwrap();
        let bin_dir = dir.path().join("Binaries").join("Win64");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("unsteam.ini"), "fake").unwrap();
        fs::write(dir.path().join("DontScream-Win64-Shipping.exe"), "fake").unwrap();

        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![dir.path().join("DontScream-Win64-Shipping.exe")],
        };
        let result = classify_candidate(&candidate, &[unsteam_signature()]);
        assert!(
            result.is_some(),
            "unsteam.ini in Binaries/Win64/ must be detected by UE deep scan"
        );
    }

    #[test]
    fn test_ue_deep_scan_finds_steam_api_in_binaries_win32_direct() {
        // Mimics Goat Simulator (after promote_out_of_bin_dir): candidate is the game root,
        // steam_api.dll is in Binaries/Win32/ directly under the candidate.
        let dir = tempfile::tempdir().unwrap();
        let bin_dir = dir.path().join("Binaries").join("Win32");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("steam_api.dll"), "fake").unwrap();
        fs::write(dir.path().join("GoatGame.exe"), "fake").unwrap();

        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![dir.path().join("GoatGame.exe")],
        };
        let sig = Signature {
            name: "Steam Generic".to_string(),
            store: Some("Steam".to_string()),
            crack_type: None,
            required_files: vec!["steam_api.dll".to_string()],
            optional_files: vec![],
            confidence_boost: vec![],
            auto_add_threshold: None,
            cleanup_files: vec![],
        };
        let result = classify_candidate(&candidate, &[sig]);
        assert!(
            result.is_some(),
            "steam_api.dll in Binaries/Win32/ must be detected by UE deep scan"
        );
    }

    #[test]
    fn test_ue_deep_scan_finds_onlinefix_ini_one_subdir_deep() {
        // Mimics KEEP GAMBLING: candidate root is the game root (e.g. KEEP GAMBLING/),
        // OnlineFix.ini is at KEEP GAMBLING/Kaching/Binaries/Win64/OnlineFix.ini.
        let dir = tempfile::tempdir().unwrap();
        let bin_dir = dir.path().join("Kaching").join("Binaries").join("Win64");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("OnlineFix.ini"), "fake").unwrap();
        fs::write(dir.path().join("launcher.exe"), "fake").unwrap();

        let candidate = GameCandidate {
            path: dir.path().to_path_buf(),
            exe_files: vec![dir.path().join("launcher.exe")],
        };
        let result = classify_candidate(&candidate, &[onlinefix_ue_signature()]);
        assert!(
            result.is_some(),
            "OnlineFix.ini in <subdir>/Binaries/Win64/ must be detected by UE deep scan"
        );
    }
}

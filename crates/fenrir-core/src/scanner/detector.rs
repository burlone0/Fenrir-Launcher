use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;
use walkdir::WalkDir;

const IGNORED_DIRS: &[&str] = &[
    "_Redist",
    "Redistributables",
    "DirectX",
    "DotNetFX",
    "__MACOSX",
    "_CommonRedist",
    "CommonRedist",
    "Redist",
    "vcredist",
    "directx",
    // Unreal engine and third-party runtime noise
    "Engine",
    "BepInEx",
    "MelonLoader",
    "RedistPackages",
    "common-redist",
    "__Installer",
    "$PLUGINSDIR",
    "DXSETUP",
    "Support",
    "Extras",
    "NativeMods",
    "PatchFiles",
];

/// Max parent levels to walk up from an exe looking for a signature marker.
const ROOT_WALKUP_MAX: usize = 4;

/// File names / globs that mark a "real" game root. Presence of any of these
/// in a directory strongly suggests that dir is the game's install root,
/// regardless of where the exe lives.
const ROOT_MARKERS: &[&str] = &[
    "steam_api.dll",
    "steam_api64.dll",
    "steam_appid.txt",
    "steam_emu.ini",
    "OnlineFix.ini",
    "OnlineFix.url",
    "OnlineFix64.dll",
    "cream_api.ini",
    "EOSSDK-Win64-Shipping.dll",
    "EOSSDK-Win32-Shipping.dll",
    "GalaxyClient.dll",
    "galaxy.dll",
    "game.id",
    "STEAMRIP \u{00BB} Free Pre-installed Steam Games.url",
];

/// Glob markers (substring-based, simple prefix+suffix match). Kept minimal
/// to avoid pulling a full glob dependency on the hot path.
const ROOT_MARKER_GLOBS: &[(&str, &str)] = &[("goggame-", ".info")];

/// Exe names (lowercase stems) that identify installers/uninstallers rather
/// than a real game binary. A candidate whose exes are *all* in this list is
/// dropped.
const INSTALLER_EXE_PREFIXES: &[&str] = &[
    "vcredist",
    "dxsetup",
    "directx",
    "dotnetfx",
    "unins",
    "setup",
    "install",
    "oalinst",
    "xnafx",
    "crashreport",
];

#[derive(Debug, Clone)]
pub struct GameCandidate {
    pub path: PathBuf,
    pub exe_files: Vec<PathBuf>,
}

pub fn find_game_candidates(root: &Path, max_depth: usize) -> Vec<GameCandidate> {
    // Step 1: walk the tree, collect (resolved_root, exe) pairs.
    // resolved_root = parent of exe, promoted up to the nearest directory
    // containing signature markers (bounded by ROOT_WALKUP_MAX).
    let mut grouped: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    for entry in WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_ignored_entry(e))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !ext.eq_ignore_ascii_case("exe") {
            continue;
        }
        let Some(parent) = path.parent() else {
            continue;
        };
        if parent == root {
            continue;
        }

        let resolved = resolve_game_root(parent, root);
        grouped
            .entry(resolved)
            .or_default()
            .push(path.to_path_buf());
    }

    // Step 2: materialize candidates.
    let mut candidates: Vec<GameCandidate> = grouped
        .into_iter()
        .map(|(path, exe_files)| GameCandidate { path, exe_files })
        .collect();

    // Step 3: dedup ancestor/descendant — if candidate A's path is an ancestor
    // of candidate B's path, B's exes move into A and B is dropped.
    candidates.sort_by(|a, b| {
        a.path
            .components()
            .count()
            .cmp(&b.path.components().count())
    });
    let mut i = 0;
    while i < candidates.len() {
        let ancestor_path = candidates[i].path.clone();
        let mut j = i + 1;
        while j < candidates.len() {
            if candidates[j].path.starts_with(&ancestor_path) {
                let absorbed = candidates.remove(j);
                candidates[i].exe_files.extend(absorbed.exe_files);
            } else {
                j += 1;
            }
        }
        i += 1;
    }

    // Step 4: drop candidates whose exes are *all* installers.
    candidates.retain(|c| !is_installer_only(c));

    debug!(
        "found {} game candidates in {}",
        candidates.len(),
        root.display()
    );
    candidates
}

/// Walk up from `start` (bounded by `scan_root` and `ROOT_WALKUP_MAX`) to find
/// the nearest directory containing a signature marker. Falls back to `start`.
fn resolve_game_root(start: &Path, scan_root: &Path) -> PathBuf {
    let mut current = start.to_path_buf();
    for _ in 0..=ROOT_WALKUP_MAX {
        if has_root_marker(&current) {
            return current;
        }
        match current.parent() {
            Some(p) if p.starts_with(scan_root) && p != scan_root => current = p.to_path_buf(),
            _ => break,
        }
    }
    start.to_path_buf()
}

fn has_root_marker(dir: &Path) -> bool {
    // Exact-name markers (case-insensitive)
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name_os = entry.file_name();
            let Some(name) = name_os.to_str() else {
                continue;
            };
            let name_lc = name.to_ascii_lowercase();
            if ROOT_MARKERS
                .iter()
                .any(|m| name_lc == m.to_ascii_lowercase())
            {
                return true;
            }
            if ROOT_MARKER_GLOBS
                .iter()
                .any(|(prefix, suffix)| name_lc.starts_with(prefix) && name_lc.ends_with(suffix))
            {
                return true;
            }
        }
    }
    false
}

fn is_installer_only(candidate: &GameCandidate) -> bool {
    if candidate.exe_files.is_empty() {
        return true;
    }
    candidate.exe_files.iter().all(|exe| {
        let Some(stem) = exe.file_stem().and_then(|s| s.to_str()) else {
            return false;
        };
        let stem_lc = stem.to_ascii_lowercase();
        INSTALLER_EXE_PREFIXES
            .iter()
            .any(|p| stem_lc.starts_with(p))
    })
}

fn is_ignored_entry(entry: &walkdir::DirEntry) -> bool {
    if entry.file_type().is_dir() {
        if let Some(name) = entry.file_name().to_str() {
            return IGNORED_DIRS.iter().any(|&d| name.eq_ignore_ascii_case(d));
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_find_candidates_with_exe() {
        let dir = tempfile::tempdir().unwrap();
        let game_dir = dir.path().join("MyGame");
        fs::create_dir(&game_dir).unwrap();
        fs::write(game_dir.join("game.exe"), "fake").unwrap();

        let candidates = find_game_candidates(dir.path(), 4);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].path, game_dir);
    }

    #[test]
    fn test_skip_redist_directories() {
        let dir = tempfile::tempdir().unwrap();
        let redist = dir.path().join("_Redist");
        fs::create_dir(&redist).unwrap();
        fs::write(redist.join("vcredist.exe"), "fake").unwrap();

        let candidates = find_game_candidates(dir.path(), 4);
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_collects_multiple_exes() {
        let dir = tempfile::tempdir().unwrap();
        let game_dir = dir.path().join("MyGame");
        fs::create_dir(&game_dir).unwrap();
        fs::write(game_dir.join("game.exe"), "fake").unwrap();
        fs::write(game_dir.join("launcher.exe"), "fake").unwrap();

        let candidates = find_game_candidates(dir.path(), 4);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].exe_files.len(), 2);
    }

    #[test]
    fn test_multiple_game_dirs() {
        let dir = tempfile::tempdir().unwrap();
        for name in &["GameA", "GameB", "GameC"] {
            let game_dir = dir.path().join(name);
            fs::create_dir(&game_dir).unwrap();
            fs::write(game_dir.join("game.exe"), "fake").unwrap();
        }

        let candidates = find_game_candidates(dir.path(), 4);
        assert_eq!(candidates.len(), 3);
    }

    // -----------------------------------------------------------------------
    // Bug #2: root identification — exe in subfolder, signature files in root
    // -----------------------------------------------------------------------

    #[test]
    fn test_root_walkup_unreal_layout() {
        // Unreal-style layout: Game/Binaries/Win64/game.exe with markers in root
        let dir = tempfile::tempdir().unwrap();
        let game_dir = dir.path().join("MyGame");
        let bin_dir = game_dir.join("Binaries").join("Win64");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("MyGame.exe"), "fake").unwrap();
        // Marker file lives in the real game root, not next to the exe
        fs::write(game_dir.join("steam_api64.dll"), "fake").unwrap();

        let candidates = find_game_candidates(dir.path(), 6);
        assert_eq!(
            candidates.len(),
            1,
            "exe in Binaries/Win64/ must resolve to the game root"
        );
        assert_eq!(candidates[0].path, game_dir);
        assert!(candidates[0]
            .exe_files
            .iter()
            .any(|e| e.ends_with("MyGame.exe")));
    }

    #[test]
    fn test_root_walkup_gog_marker() {
        let dir = tempfile::tempdir().unwrap();
        let game_dir = dir.path().join("Witcher3");
        let bin_dir = game_dir.join("bin").join("x64");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("witcher3.exe"), "fake").unwrap();
        fs::write(game_dir.join("goggame-1207664643.info"), "fake").unwrap();

        let candidates = find_game_candidates(dir.path(), 6);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].path, game_dir);
    }

    #[test]
    fn test_root_walkup_no_markers_falls_back_to_parent() {
        // No signature markers anywhere: the immediate parent wins
        let dir = tempfile::tempdir().unwrap();
        let game_dir = dir.path().join("OddGame");
        let bin_dir = game_dir.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("game.exe"), "fake").unwrap();

        let candidates = find_game_candidates(dir.path(), 6);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].path, bin_dir);
    }

    // -----------------------------------------------------------------------
    // Bug #1: dedup parent/child and filter installer-only candidates
    // -----------------------------------------------------------------------

    #[test]
    fn test_dedup_parent_and_nested_exe() {
        let dir = tempfile::tempdir().unwrap();
        let game_dir = dir.path().join("Game");
        fs::create_dir(&game_dir).unwrap();
        fs::write(game_dir.join("game.exe"), "fake").unwrap();
        fs::write(game_dir.join("steam_api.dll"), "fake").unwrap();
        let tools = game_dir.join("tools");
        fs::create_dir(&tools).unwrap();
        fs::write(tools.join("helper.exe"), "fake").unwrap();

        let candidates = find_game_candidates(dir.path(), 6);
        assert_eq!(
            candidates.len(),
            1,
            "nested exe must merge into parent candidate"
        );
        assert_eq!(candidates[0].path, game_dir);
        assert_eq!(candidates[0].exe_files.len(), 2);
    }

    #[test]
    fn test_installer_only_candidate_is_filtered() {
        let dir = tempfile::tempdir().unwrap();
        let odd = dir.path().join("SetupBundle");
        fs::create_dir(&odd).unwrap();
        fs::write(odd.join("vcredist_x64.exe"), "fake").unwrap();
        fs::write(odd.join("dxsetup.exe"), "fake").unwrap();
        fs::write(odd.join("unins000.exe"), "fake").unwrap();

        let candidates = find_game_candidates(dir.path(), 4);
        assert!(
            candidates.is_empty(),
            "installer-only folder must not be a candidate"
        );
    }

    #[test]
    fn test_engine_subdir_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let game = dir.path().join("MyGame");
        let engine = game.join("Engine").join("Binaries").join("Win64");
        fs::create_dir_all(&engine).unwrap();
        fs::write(engine.join("CrashReportClient.exe"), "fake").unwrap();
        let real_bin = game.join("MyGame").join("Binaries").join("Win64");
        fs::create_dir_all(&real_bin).unwrap();
        fs::write(real_bin.join("MyGame.exe"), "fake").unwrap();
        fs::write(game.join("steam_api64.dll"), "fake").unwrap();

        let candidates = find_game_candidates(dir.path(), 8);
        assert_eq!(
            candidates.len(),
            1,
            "Engine/ must be ignored, only game root remains"
        );
        assert_eq!(candidates[0].path, game);
    }

    #[test]
    fn test_steamrip_url_promotes_to_parent_as_game_root() {
        let dir = tempfile::tempdir().unwrap();
        let parent = dir.path().join("Keep-Talking-SteamRIP.com");
        let subdir = parent.join("KTANE.v1.9.24");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(
            parent.join("STEAMRIP \u{00BB} Free Pre-installed Steam Games.url"),
            "[InternetShortcut]\nURL=https://steamrip.com",
        )
        .unwrap();
        fs::write(subdir.join("ktane.exe"), "fake").unwrap();

        let candidates = find_game_candidates(dir.path(), 6);
        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].path, parent,
            "game root must be promoted to the parent containing the SteamRIP URL"
        );
    }
}

use std::collections::HashSet;
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
    "Redist",
    "vcredist",
    "directx",
];

#[derive(Debug, Clone)]
pub struct GameCandidate {
    pub path: PathBuf,
    pub exe_files: Vec<PathBuf>,
}

pub fn find_game_candidates(root: &Path, max_depth: usize) -> Vec<GameCandidate> {
    let mut candidates: Vec<GameCandidate> = Vec::new();
    let mut seen_dirs: HashSet<PathBuf> = HashSet::new();

    for entry in WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_ignored_entry(e))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("exe") {
                    if let Some(parent) = path.parent() {
                        if parent != root {
                            if seen_dirs.insert(parent.to_path_buf()) {
                                candidates.push(GameCandidate {
                                    path: parent.to_path_buf(),
                                    exe_files: vec![path.to_path_buf()],
                                });
                            } else if let Some(c) =
                                candidates.iter_mut().find(|c| c.path == parent)
                            {
                                c.exe_files.push(path.to_path_buf());
                            }
                        }
                    }
                }
            }
        }
    }

    debug!(
        "found {} game candidates in {}",
        candidates.len(),
        root.display()
    );
    candidates
}

fn is_ignored_entry(entry: &walkdir::DirEntry) -> bool {
    if entry.file_type().is_dir() {
        if let Some(name) = entry.file_name().to_str() {
            return IGNORED_DIRS
                .iter()
                .any(|&d| name.eq_ignore_ascii_case(d));
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
}

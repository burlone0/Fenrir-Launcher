use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Patterns that are categorically unsafe — never accept them.
const BLOCKED_PATTERNS: &[&str] = &["*", "**", "../*", "**/*"];

pub struct CleanupEntry {
    pub path: PathBuf,
    pub is_dir: bool,
}

pub struct CleanupPlan {
    pub entries: Vec<CleanupEntry>,
}

impl CleanupPlan {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn file_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.is_dir).count()
    }

    pub fn dir_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_dir).count()
    }

    pub fn total_size_bytes(&self) -> u64 {
        self.entries
            .iter()
            .map(|e| {
                if e.is_dir {
                    dir_size(&e.path)
                } else {
                    e.path.metadata().map(|m| m.len()).unwrap_or(0)
                }
            })
            .sum()
    }
}

pub struct CleanupResult {
    pub removed: usize,
    pub errors: usize,
}

/// Resolves `patterns` relative to `install_dir` into a plan of paths to remove.
/// All paths are canonicalized and verified to be inside `install_dir`.
pub fn build_cleanup_plan(install_dir: &Path, patterns: &[String]) -> CleanupPlan {
    let canonical_install = match install_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return CleanupPlan { entries: vec![] },
    };

    let mut entries = Vec::new();

    for pattern in patterns {
        if pattern.contains("..") || pattern.starts_with('/') {
            warn!("rejected unsafe cleanup pattern: {:?}", pattern);
            continue;
        }
        if BLOCKED_PATTERNS.contains(&pattern.as_str()) {
            warn!("rejected blocked cleanup pattern: {:?}", pattern);
            continue;
        }

        if pattern.ends_with('/') {
            let dir_name = pattern.trim_end_matches('/');
            let path = install_dir.join(dir_name);
            if path.is_dir() {
                if let Ok(canonical) = path.canonicalize() {
                    if canonical.starts_with(&canonical_install) && canonical != canonical_install {
                        entries.push(CleanupEntry {
                            path: canonical,
                            is_dir: true,
                        });
                    } else {
                        warn!(
                            "cleanup dir escapes install_dir, skipped: {}",
                            path.display()
                        );
                    }
                }
            }
        } else if pattern.contains('*') {
            let glob_pattern = format!("{}/{}", canonical_install.display(), pattern);
            match glob::glob(&glob_pattern) {
                Ok(paths) => {
                    for result in paths {
                        let p = match result {
                            Ok(p) => p,
                            Err(_) => continue,
                        };
                        if !p.exists() {
                            continue;
                        }
                        let is_dir = p.is_dir();
                        if let Ok(canonical) = p.canonicalize() {
                            if canonical.starts_with(&canonical_install)
                                && canonical != canonical_install
                            {
                                entries.push(CleanupEntry {
                                    path: canonical,
                                    is_dir,
                                });
                            }
                        }
                    }
                }
                Err(e) => warn!("invalid glob pattern {:?}: {}", pattern, e),
            }
        } else {
            let path = install_dir.join(pattern);
            if path.exists() {
                let is_dir = path.is_dir();
                if let Ok(canonical) = path.canonicalize() {
                    if canonical.starts_with(&canonical_install) && canonical != canonical_install {
                        entries.push(CleanupEntry {
                            path: canonical,
                            is_dir,
                        });
                    } else {
                        warn!(
                            "cleanup path escapes install_dir, skipped: {}",
                            path.display()
                        );
                    }
                }
            }
        }
    }

    // Deduplicate: drop entries that are inside an already-listed directory
    entries.sort_by(|a, b| {
        a.path
            .components()
            .count()
            .cmp(&b.path.components().count())
    });
    entries.dedup_by(|later, earlier| later.path.starts_with(&earlier.path) && earlier.is_dir);

    CleanupPlan { entries }
}

/// Removes all entries in `plan`. Returns `(removed, errors)`.
/// A failure on one entry does not abort the rest.
pub fn execute_cleanup(plan: &CleanupPlan) -> CleanupResult {
    let mut removed = 0;
    let mut errors = 0;

    for entry in &plan.entries {
        if entry.is_dir {
            match std::fs::remove_dir_all(&entry.path) {
                Ok(()) => {
                    info!("removed dir: {}", entry.path.display());
                    removed += 1;
                }
                Err(e) => {
                    warn!("failed to remove dir {}: {}", entry.path.display(), e);
                    errors += 1;
                }
            }
        } else {
            match std::fs::remove_file(&entry.path) {
                Ok(()) => {
                    info!("removed file: {}", entry.path.display());
                    removed += 1;
                }
                Err(e) => {
                    warn!("failed to remove file {}: {}", entry.path.display(), e);
                    errors += 1;
                }
            }
        }
    }

    CleanupResult { removed, errors }
}

fn dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_build_plan_exact_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("OnlineFix.url"), "fake").unwrap();

        let plan = build_cleanup_plan(dir.path(), &["OnlineFix.url".to_string()]);
        assert_eq!(plan.file_count(), 1);
        assert_eq!(plan.dir_count(), 0);
    }

    #[test]
    fn test_build_plan_directory() {
        let dir = tempfile::tempdir().unwrap();
        let redist = dir.path().join("_Redist");
        fs::create_dir(&redist).unwrap();
        fs::write(redist.join("vcredist.exe"), "fake").unwrap();

        let plan = build_cleanup_plan(dir.path(), &["_Redist/".to_string()]);
        assert_eq!(plan.dir_count(), 1);
        assert_eq!(plan.file_count(), 0);
    }

    #[test]
    fn test_build_plan_glob() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("OnlineFix.url"), "fake").unwrap();
        fs::write(dir.path().join("fitgirl-repacks.site.url"), "fake").unwrap();

        let plan = build_cleanup_plan(dir.path(), &["*.url".to_string()]);
        assert_eq!(plan.file_count(), 2);
    }

    #[test]
    fn test_unsafe_pattern_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let plan = build_cleanup_plan(dir.path(), &["../etc/passwd".to_string()]);
        assert!(plan.is_empty(), "path traversal must be rejected");
    }

    #[test]
    fn test_absolute_pattern_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let plan = build_cleanup_plan(dir.path(), &["/etc/passwd".to_string()]);
        assert!(plan.is_empty(), "absolute paths must be rejected");
    }

    #[test]
    fn test_wildcard_only_pattern_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let plan = build_cleanup_plan(dir.path(), &["*".to_string()]);
        assert!(plan.is_empty(), "bare wildcard must be rejected");
    }

    #[test]
    fn test_missing_file_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let plan = build_cleanup_plan(dir.path(), &["nonexistent.url".to_string()]);
        assert!(plan.is_empty(), "missing files must not appear in plan");
    }

    #[test]
    fn test_execute_cleanup_removes_entries() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("noise.url"), "fake").unwrap();
        let subdir = dir.path().join("_Redist");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("vcredist.exe"), "fake").unwrap();

        let plan = build_cleanup_plan(
            dir.path(),
            &["noise.url".to_string(), "_Redist/".to_string()],
        );
        assert_eq!(plan.file_count() + plan.dir_count(), 2);

        let result = execute_cleanup(&plan);
        assert_eq!(result.removed, 2);
        assert_eq!(result.errors, 0);
        assert!(!dir.path().join("noise.url").exists());
        assert!(!subdir.exists());
    }

    #[test]
    fn test_execute_cleanup_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("noise.url"), "fake").unwrap();

        let plan = build_cleanup_plan(dir.path(), &["noise.url".to_string()]);
        execute_cleanup(&plan);

        // Second plan must be empty since file is already gone
        let plan2 = build_cleanup_plan(dir.path(), &["noise.url".to_string()]);
        assert!(plan2.is_empty(), "second plan must be empty (idempotent)");
    }

    #[test]
    fn test_dedup_file_inside_listed_dir() {
        // If both "_Redist/" and "_Redist/vcredist.exe" are in patterns,
        // only the directory entry should remain (the file is inside it).
        let dir = tempfile::tempdir().unwrap();
        let redist = dir.path().join("_Redist");
        fs::create_dir(&redist).unwrap();
        fs::write(redist.join("vcredist.exe"), "fake").unwrap();

        let plan = build_cleanup_plan(
            dir.path(),
            &["_Redist/".to_string(), "_Redist/vcredist.exe".to_string()],
        );
        assert_eq!(plan.dir_count(), 1, "directory entry must be kept");
        assert_eq!(plan.file_count(), 0, "child file must be deduped");
    }
}

pub mod classifier;
pub mod detector;
pub mod signatures;

use crate::error::ScannerError;
use classifier::ClassifiedGame;
use signatures::Signature;
use std::path::Path;
use tracing::info;

#[derive(Debug)]
pub struct ScanResult {
    pub high_confidence: Vec<ClassifiedGame>,
    pub needs_confirmation: Vec<ClassifiedGame>,
    pub total_candidates: usize,
}

pub fn scan_directory(
    root: &Path,
    signatures: &[Signature],
    max_depth: usize,
) -> Result<ScanResult, ScannerError> {
    if !root.exists() {
        return Err(ScannerError::DirNotFound(root.to_path_buf()));
    }

    let candidates = detector::find_game_candidates(root, max_depth);
    let total_candidates = candidates.len();
    info!(
        "found {} candidates in {}",
        total_candidates,
        root.display()
    );

    let mut high_confidence = Vec::new();
    let mut needs_confirmation = Vec::new();

    for candidate in &candidates {
        if let Some((score, classified)) = classifier::classify_candidate(candidate, signatures) {
            if score >= classified.high_confidence_threshold {
                high_confidence.push(classified);
            } else {
                needs_confirmation.push(classified);
            }
        }
    }

    info!(
        "classified: {} high confidence, {} needs confirmation",
        high_confidence.len(),
        needs_confirmation.len()
    );

    Ok(ScanResult {
        high_confidence,
        needs_confirmation,
        total_candidates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_nonexistent_dir_returns_error() {
        let result = scan_directory(std::path::Path::new("/nonexistent/path/abc"), &[], 4);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("scan directory not found"));
    }
}

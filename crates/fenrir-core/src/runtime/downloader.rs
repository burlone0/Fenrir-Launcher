use crate::error::DownloadError;
use crate::runtime::github::GitHubRelease;
use futures_util::StreamExt;
use sha2::{Digest, Sha512};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

/// Callback for progress reporting (downloaded_bytes, total_bytes).
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

/// Compute SHA-512 hash of a byte slice.
pub fn compute_sha512(data: &[u8]) -> String {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Verify SHA-512 hash matches expected.
pub fn verify_sha512(data: &[u8], expected: &str) -> bool {
    compute_sha512(data) == expected
}

/// Parse a .sha512sum file to extract the hash for a specific filename.
pub fn parse_checksum_file(content: &str, filename: &str) -> Option<String> {
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == filename {
            return Some(parts[0].to_string());
        }
    }
    None
}

/// Download a file from a URL with optional progress reporting.
pub async fn download_file(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    progress: Option<&ProgressCallback>,
) -> Result<(), DownloadError> {
    info!("downloading {} to {}", url, dest.display());

    let response = client
        .get(url)
        .header("User-Agent", "fenrir-launcher")
        .send()
        .await?;

    let total_size = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).await?;
    }

    let mut file = fs::File::create(dest).await?;
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        if let Some(cb) = &progress {
            cb(downloaded, total_size);
        }
    }

    file.flush().await?;
    debug!("download complete: {} bytes", downloaded);
    Ok(())
}

/// Full runtime download: tarball + checksum verification + extraction.
pub async fn download_runtime(
    client: &reqwest::Client,
    release: &GitHubRelease,
    runtime_dir: &Path,
    progress: Option<ProgressCallback>,
) -> Result<PathBuf, DownloadError> {
    let tarball = release
        .find_tarball()
        .ok_or_else(|| DownloadError::NoTarball(release.tag_name.clone()))?;

    let temp_dir = runtime_dir.join(".tmp");
    fs::create_dir_all(&temp_dir).await?;

    let tarball_path = temp_dir.join(&tarball.name);

    // 1. Download tarball
    download_file(
        client,
        &tarball.browser_download_url,
        &tarball_path,
        progress.as_ref(),
    )
    .await?;

    // 2. Verify checksum if available
    if let Some(checksum_asset) = release.find_checksum() {
        info!("verifying checksum...");
        let checksum_path = temp_dir.join(&checksum_asset.name);
        download_file(
            client,
            &checksum_asset.browser_download_url,
            &checksum_path,
            None,
        )
        .await?;

        let checksum_content = fs::read_to_string(&checksum_path).await?;
        let tarball_data = fs::read(&tarball_path).await?;

        if let Some(expected_hash) = parse_checksum_file(&checksum_content, &tarball.name) {
            let actual_hash = compute_sha512(&tarball_data);
            if actual_hash != expected_hash {
                let _ = fs::remove_dir_all(&temp_dir).await;
                return Err(DownloadError::ChecksumMismatch {
                    expected: expected_hash,
                    actual: actual_hash,
                });
            }
            info!("checksum verified");
        }

        let _ = fs::remove_file(&checksum_path).await;
    }

    // 3. Extract tarball
    let dest = runtime_dir.join(&release.tag_name);
    info!("extracting to {}...", dest.display());

    let tarball_path_clone = tarball_path.clone();
    let runtime_dir_owned = runtime_dir.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(&tarball_path_clone).map_err(DownloadError::Io)?;
        let decompressed = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decompressed);
        archive
            .unpack(&runtime_dir_owned)
            .map_err(|e| DownloadError::Extraction(e.to_string()))?;
        Ok::<(), DownloadError>(())
    })
    .await
    .map_err(|e| DownloadError::Extraction(e.to_string()))??;

    // 4. Cleanup temp
    let _ = fs::remove_dir_all(&temp_dir).await;

    info!("runtime installed: {}", release.tag_name);
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_sha512_correct() {
        let data = b"test data for checksum";
        let hash = compute_sha512(data);
        assert!(verify_sha512(data, &hash));
    }

    #[test]
    fn test_verify_sha512_incorrect() {
        let data = b"test data";
        assert!(!verify_sha512(data, "0000000000000000"));
    }

    #[test]
    fn test_parse_checksum_file() {
        let content = "abc123def456  GE-Proton9-20.tar.gz\n";
        let hash = parse_checksum_file(content, "GE-Proton9-20.tar.gz");
        assert_eq!(hash, Some("abc123def456".to_string()));
    }

    #[test]
    fn test_parse_checksum_file_not_found() {
        let content = "abc123def456  other-file.tar.gz\n";
        let hash = parse_checksum_file(content, "GE-Proton9-20.tar.gz");
        assert!(hash.is_none());
    }

    #[test]
    fn test_parse_checksum_multiple_lines() {
        let content = "aaa111  file-a.tar.gz\nbbb222  file-b.tar.gz\n";
        assert_eq!(
            parse_checksum_file(content, "file-b.tar.gz"),
            Some("bbb222".to_string())
        );
    }

    #[test]
    fn test_compute_sha512_deterministic() {
        let hash1 = compute_sha512(b"hello");
        let hash2 = compute_sha512(b"hello");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, compute_sha512(b"world"));
    }

    #[test]
    fn test_parse_checksum_empty_content() {
        assert!(parse_checksum_file("", "any.tar.gz").is_none());
    }

    #[test]
    fn test_parse_checksum_malformed_line() {
        // Line with only one token — must not match
        let content = "onlyone\n";
        assert!(parse_checksum_file(content, "onlyone").is_none());
    }

    #[tokio::test]
    async fn test_download_runtime_no_tarball_returns_error() {
        let client = reqwest::Client::new();
        let release = crate::runtime::github::GitHubRelease {
            tag_name: "test-release".to_string(),
            name: "test-release".to_string(),
            assets: vec![],
        };
        let dir = tempfile::tempdir().unwrap();
        let result = download_runtime(&client, &release, dir.path(), None).await;
        assert!(
            matches!(result, Err(DownloadError::NoTarball(_))),
            "expected NoTarball error, got: {:?}",
            result
        );
    }
}

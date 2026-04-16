use serde::Deserialize;
use tracing::info;

const PROTON_GE_REPO: &str = "GloriousEggroll/proton-ge-custom";
const WINE_GE_REPO: &str = "GloriousEggroll/wine-ge-custom";

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

impl GitHubRelease {
    pub fn find_tarball(&self) -> Option<&GitHubAsset> {
        self.assets
            .iter()
            .find(|a| a.name.ends_with(".tar.gz") || a.name.ends_with(".tar.xz"))
    }

    pub fn find_checksum(&self) -> Option<&GitHubAsset> {
        self.assets
            .iter()
            .find(|a| a.name.ends_with(".sha512sum") || a.name.ends_with(".sha256sum"))
    }
}

pub async fn list_releases(
    client: &reqwest::Client,
    repo: &str,
    limit: usize,
) -> Result<Vec<GitHubRelease>, reqwest::Error> {
    let url = format!(
        "https://api.github.com/repos/{}/releases?per_page={}",
        repo, limit
    );

    info!("fetching releases from {}", url);

    let releases = client
        .get(&url)
        .header("User-Agent", "fenrir-launcher")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .json::<Vec<GitHubRelease>>()
        .await?;

    Ok(releases)
}

pub async fn list_proton_ge_releases(
    client: &reqwest::Client,
    limit: usize,
) -> Result<Vec<GitHubRelease>, reqwest::Error> {
    list_releases(client, PROTON_GE_REPO, limit).await
}

pub async fn list_wine_ge_releases(
    client: &reqwest::Client,
    limit: usize,
) -> Result<Vec<GitHubRelease>, reqwest::Error> {
    list_releases(client, WINE_GE_REPO, limit).await
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOCK_RELEASE: &str = r#"{
        "tag_name": "GE-Proton9-20",
        "name": "GE-Proton9-20",
        "assets": [
            {
                "name": "GE-Proton9-20.tar.gz",
                "browser_download_url": "https://example.com/GE-Proton9-20.tar.gz",
                "size": 419430400
            },
            {
                "name": "GE-Proton9-20.sha512sum",
                "browser_download_url": "https://example.com/GE-Proton9-20.sha512sum",
                "size": 128
            }
        ]
    }"#;

    #[test]
    fn test_parse_release() {
        let release: GitHubRelease = serde_json::from_str(MOCK_RELEASE).unwrap();
        assert_eq!(release.tag_name, "GE-Proton9-20");
        assert_eq!(release.assets.len(), 2);
    }

    #[test]
    fn test_find_tarball_asset() {
        let release: GitHubRelease = serde_json::from_str(MOCK_RELEASE).unwrap();
        let tarball = release.find_tarball();
        assert!(tarball.is_some());
        assert!(tarball.unwrap().name.ends_with(".tar.gz"));
    }

    #[test]
    fn test_find_checksum_asset() {
        let release: GitHubRelease = serde_json::from_str(MOCK_RELEASE).unwrap();
        let checksum = release.find_checksum();
        assert!(checksum.is_some());
        assert!(checksum.unwrap().name.ends_with(".sha512sum"));
    }

    #[test]
    fn test_release_without_tarball() {
        let json = r#"{
            "tag_name": "test",
            "name": "test",
            "assets": [{"name": "readme.md", "browser_download_url": "https://x.com/r", "size": 10}]
        }"#;
        let release: GitHubRelease = serde_json::from_str(json).unwrap();
        assert!(release.find_tarball().is_none());
    }

    #[test]
    fn test_find_tarball_xz() {
        let json = r#"{
            "tag_name": "Wine-GE-Proton8-26",
            "name": "Wine-GE-Proton8-26",
            "assets": [{"name": "wine-ge-8-26-x86_64.tar.xz", "browser_download_url": "https://x.com/t", "size": 200}]
        }"#;
        let release: GitHubRelease = serde_json::from_str(json).unwrap();
        let tarball = release.find_tarball();
        assert!(tarball.is_some());
        assert!(tarball.unwrap().name.ends_with(".tar.xz"));
    }

    #[test]
    fn test_find_checksum_sha256() {
        let json = r#"{
            "tag_name": "test",
            "name": "test",
            "assets": [{"name": "file.sha256sum", "browser_download_url": "https://x.com/c", "size": 64}]
        }"#;
        let release: GitHubRelease = serde_json::from_str(json).unwrap();
        let checksum = release.find_checksum();
        assert!(checksum.is_some());
        assert!(checksum.unwrap().name.ends_with(".sha256sum"));
    }

    #[test]
    fn test_release_without_checksum() {
        let json = r#"{
            "tag_name": "test",
            "name": "test",
            "assets": [{"name": "file.tar.gz", "browser_download_url": "https://x.com/t", "size": 100}]
        }"#;
        let release: GitHubRelease = serde_json::from_str(json).unwrap();
        assert!(release.find_checksum().is_none());
    }

    #[test]
    fn test_release_empty_assets() {
        let json = r#"{"tag_name": "v1", "name": "v1", "assets": []}"#;
        let release: GitHubRelease = serde_json::from_str(json).unwrap();
        assert!(release.find_tarball().is_none());
        assert!(release.find_checksum().is_none());
    }

    #[test]
    fn test_github_asset_fields() {
        let release: GitHubRelease = serde_json::from_str(MOCK_RELEASE).unwrap();
        let asset = release.find_tarball().unwrap();
        assert_eq!(asset.size, 419430400);
        assert_eq!(
            asset.browser_download_url,
            "https://example.com/GE-Proton9-20.tar.gz"
        );
    }

    #[test]
    fn test_parse_multiple_releases() {
        let json = format!("[{}, {}]", MOCK_RELEASE, MOCK_RELEASE);
        let releases: Vec<GitHubRelease> = serde_json::from_str(&json).unwrap();
        assert_eq!(releases.len(), 2);
        assert_eq!(releases[0].tag_name, "GE-Proton9-20");
    }
}

use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::runtime;
use fenrir_core::runtime::github;
use indicatif::{ProgressBar, ProgressStyle};

pub fn list() -> Result<(), Box<dyn std::error::Error>> {
    let config = FenrirConfig::load()?;
    let runtimes = runtime::discover_all(&config.general.runtime_dir);

    if runtimes.is_empty() {
        println!("no runtimes found.");
        println!("install Wine or run 'fenrir runtime install <version>' to download GE-Proton.");
        return Ok(());
    }

    println!(
        "{:<25} {:<12} {:<10} {:<12} PATH",
        "ID", "TYPE", "VERSION", "SOURCE"
    );
    println!("{}", "-".repeat(80));

    for rt in &runtimes {
        println!(
            "{:<25} {:<12} {:<10} {:<12} {}",
            rt.id,
            rt.runtime_type,
            rt.version,
            rt.source,
            rt.path.display(),
        );
    }

    Ok(())
}

pub fn available(kind: &str) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let client = reqwest::Client::new();

        let releases = match kind {
            "proton-ge" | "proton" => github::list_proton_ge_releases(&client, 10).await?,
            "wine-ge" | "wine" => github::list_wine_ge_releases(&client, 10).await?,
            _ => {
                eprintln!(
                    "unknown runtime kind: '{}'. Use 'proton-ge' or 'wine-ge'.",
                    kind
                );
                return Ok(());
            }
        };

        if releases.is_empty() {
            println!("no releases found.");
            return Ok(());
        }

        println!("{:<30} SIZE", "VERSION");
        println!("{}", "-".repeat(50));

        for release in &releases {
            let size = release
                .find_tarball()
                .map(|a| format_size(a.size))
                .unwrap_or_else(|| "-".to_string());
            println!("{:<30} {}", release.tag_name, size);
        }

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

pub fn install(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = FenrirConfig::load()?;
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let client = reqwest::Client::new();

        // Search both repos for the version
        let mut found_release = None;
        for releases in [
            github::list_proton_ge_releases(&client, 30)
                .await
                .unwrap_or_default(),
            github::list_wine_ge_releases(&client, 30)
                .await
                .unwrap_or_default(),
        ] {
            if let Some(r) = releases.into_iter().find(|r| r.tag_name == version) {
                found_release = Some(r);
                break;
            }
        }

        let release = match found_release {
            Some(r) => r,
            None => {
                eprintln!(
                    "version '{}' not found. Run 'fenrir runtime available' to see options.",
                    version
                );
                return Ok(());
            }
        };

        let tarball = release
            .find_tarball()
            .ok_or("no tarball found in release")?;

        let pb = ProgressBar::new(tarball.size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{wide_bar} {bytes}/{total_bytes} ({eta})")
                .unwrap(),
        );
        pb.set_message(format!("downloading {}...", version));

        let progress: fenrir_core::runtime::downloader::ProgressCallback =
            Box::new(move |downloaded: u64, _total: u64| {
                pb.set_position(downloaded);
            });

        let dest = fenrir_core::runtime::downloader::download_runtime(
            &client,
            &release,
            &config.general.runtime_dir,
            Some(progress),
        )
        .await?;

        println!("installed {} at {}", version, dest.display());
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

pub fn set_default(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = FenrirConfig::load()?;
    let runtimes = runtime::discover_all(&config.general.runtime_dir);

    if runtimes.iter().any(|r| r.id == id) {
        let mut config = config;
        config.defaults.runtime = id.to_string();
        config.save()?;
        println!("default runtime set to: {}", id);
    } else {
        eprintln!("runtime '{}' not found.", id);
        if !runtimes.is_empty() {
            eprintln!(
                "available: {}",
                runtimes
                    .iter()
                    .map(|r| r.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    }
}

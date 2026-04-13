use fenrir_core::config::settings::FenrirConfig;
use fenrir_core::runtime;

pub fn list() -> Result<(), Box<dyn std::error::Error>> {
    let config = FenrirConfig::load()?;
    let runtimes = runtime::discover_all(&config.general.runtime_dir);

    if runtimes.is_empty() {
        println!("no runtimes found.");
        println!("install Wine or GE-Proton, then run 'fenrir runtime list' again.");
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

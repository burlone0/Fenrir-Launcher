use fenrir_core::config::settings::FenrirConfig;

pub fn run(
    set: Option<String>,
    value: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = FenrirConfig::load()?;

    match (set, value) {
        (Some(key), Some(val)) => {
            match key.as_str() {
                "scan.game_dirs" => {
                    let dirs: Vec<std::path::PathBuf> =
                        val.split(',').map(|s| std::path::PathBuf::from(s.trim())).collect();
                    config.scan.game_dirs = dirs;
                }
                "scan.auto_scan" => config.scan.auto_scan = val.parse()?,
                "privacy.fetch_metadata" => config.privacy.fetch_metadata = val.parse()?,
                "privacy.fetch_covers" => config.privacy.fetch_covers = val.parse()?,
                "defaults.runtime" => config.defaults.runtime = val.clone(),
                "defaults.enable_dxvk" => config.defaults.enable_dxvk = val.parse()?,
                "defaults.enable_vkd3d" => config.defaults.enable_vkd3d = val.parse()?,
                "defaults.esync" => config.defaults.esync = val.parse()?,
                "defaults.fsync" => config.defaults.fsync = val.parse()?,
                _ => {
                    eprintln!("unknown config key: {}", key);
                    return Ok(());
                }
            }
            config.save()?;
            println!("config updated: {} = {}", key, val);
        }
        _ => {
            let toml_str = toml::to_string_pretty(&config)?;
            println!("{}", toml_str);
        }
    }

    Ok(())
}

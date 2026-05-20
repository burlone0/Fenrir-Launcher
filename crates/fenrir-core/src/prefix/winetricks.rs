use crate::error::PrefixError;
use crate::prefix::profile::WinetricksConfig;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info, warn};

/// Install the components a profile requires into the given prefix.
///
/// - `components` are mandatory: a failure returns `PrefixError::WinetricksComponent`.
/// - `optional` are best-effort: failures are logged at warn level and skipped.
/// - Already-installed components are detected via `winetricks list-installed` and skipped.
///
/// `on_step` is called with a human-readable progress string before each component
/// install attempt — wire it into the GUI's `configure:step` events or the CLI's
/// stdout.
///
/// Returns `Ok(())` early (without invoking winetricks) when the config has nothing
/// to install, so it is safe to call unconditionally from the configure flow.
pub fn install_components<F>(
    prefix_path: &Path,
    config: &WinetricksConfig,
    mut on_step: F,
) -> Result<(), PrefixError>
where
    F: FnMut(&str),
{
    if config.components.is_empty() && config.optional.is_empty() {
        return Ok(());
    }

    if !is_winetricks_available() {
        return Err(PrefixError::WinetricksMissing);
    }

    let installed = list_installed_components(prefix_path).unwrap_or_else(|e| {
        debug!("winetricks list-installed failed ({e}), assuming nothing installed");
        Vec::new()
    });

    for component in &config.components {
        if installed.iter().any(|i| i == component) {
            on_step(&format!("{component} already installed, skipping"));
            debug!(
                "winetricks: {component} already installed in {}",
                prefix_path.display()
            );
            continue;
        }
        on_step(&format!(
            "installing {component} (this may take several minutes)"
        ));
        info!(
            "winetricks: installing {component} into {}",
            prefix_path.display()
        );
        install_one(prefix_path, component).map_err(|reason| PrefixError::WinetricksComponent {
            component: component.clone(),
            reason,
        })?;
    }

    for component in &config.optional {
        if installed.iter().any(|i| i == component) {
            continue;
        }
        on_step(&format!("installing optional {component}"));
        if let Err(reason) = install_one(prefix_path, component) {
            warn!("optional winetricks component {component} failed: {reason} — continuing");
        }
    }

    Ok(())
}

/// `winetricks --version` returns ENOENT when the binary is missing; any
/// successful spawn means it exists.
fn is_winetricks_available() -> bool {
    Command::new("winetricks").arg("--version").output().is_ok()
}

fn list_installed_components(prefix_path: &Path) -> std::io::Result<Vec<String>> {
    let output = Command::new("winetricks")
        .env("WINEPREFIX", prefix_path)
        .arg("list-installed")
        .output()?;
    if !output.status.success() {
        // Fresh prefixes return non-zero from list-installed; treat as empty.
        return Ok(Vec::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

fn install_one(prefix_path: &Path, component: &str) -> Result<(), String> {
    let status = Command::new("winetricks")
        .env("WINEPREFIX", prefix_path)
        .args(["-q", component])
        .status()
        .map_err(|e| format!("failed to spawn winetricks: {e}"))?;
    if !status.success() {
        return Err(format!(
            "winetricks exited with {}",
            status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "no exit code".into())
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_install_components_empty_config_returns_ok() {
        // Must NOT invoke winetricks when there is nothing to install — safe
        // to call unconditionally on every configure.
        let dir = tempdir().unwrap();
        let config = WinetricksConfig::default();
        let mut called = false;
        let result = install_components(dir.path(), &config, |_| {
            called = true;
        });
        assert!(result.is_ok());
        assert!(!called, "on_step must not be invoked when config is empty");
    }

    #[test]
    fn test_install_components_missing_winetricks_returns_error_when_components_listed() {
        // We cannot reliably mock PATH inside a test, so this asserts the error
        // shape only if winetricks really is missing on the test runner. CI uses
        // a minimal image without winetricks, dev machines often have it.
        if is_winetricks_available() {
            return;
        }
        let dir = tempdir().unwrap();
        let config = WinetricksConfig {
            components: vec!["dotnetdesktop6".into()],
            optional: vec![],
        };
        let result = install_components(dir.path(), &config, |_| {});
        assert!(matches!(result, Err(PrefixError::WinetricksMissing)));
    }
}

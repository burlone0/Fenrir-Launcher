use crate::error::PrefixError;
use crate::prefix::builder;
use crate::prefix::profile::WineProfile;
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

/// Result of applying a profile to a prefix.
pub struct TuneResult {
    pub env_vars: HashMap<String, String>,
    pub profile_name: String,
}

/// Apply a complete Wine profile to a prefix.
/// User overrides always take priority over profile defaults.
pub fn apply_profile(
    prefix_path: &Path,
    wine_binary: &Path,
    profile: &WineProfile,
    user_overrides: Option<&serde_json::Value>,
) -> Result<TuneResult, PrefixError> {
    info!(
        "applying profile '{}' to {}",
        profile.profile.name,
        prefix_path.display()
    );

    // 1. DLL overrides: profile base + user additions
    let mut dll_overrides = profile.wine.dll_overrides.clone();
    if let Some(overrides) = user_overrides {
        if let Some(user_dlls) = overrides.get("dll_overrides").and_then(|v| v.as_array()) {
            for dll in user_dlls {
                if let Some(s) = dll.as_str() {
                    dll_overrides.push(s.to_string());
                }
            }
        }
    }
    builder::set_dll_overrides(prefix_path, wine_binary, &dll_overrides)?;

    // 2. Build env vars
    let mut env =
        builder::build_wine_env(prefix_path, profile.features.esync, profile.features.fsync);

    // Add profile env vars
    for (k, v) in &profile.env {
        env.insert(k.clone(), v.clone());
    }

    // User override env vars win
    if let Some(overrides) = user_overrides {
        if let Some(user_env) = overrides.get("env_vars").and_then(|v| v.as_object()) {
            for (k, v) in user_env {
                if let Some(s) = v.as_str() {
                    env.insert(k.clone(), s.to_string());
                }
            }
        }
    }

    Ok(TuneResult {
        env_vars: env,
        profile_name: profile.profile.name.clone(),
    })
}

pub mod builder;
pub mod profile;
pub mod tuner;

pub use builder::{build_wine_env, create_prefix, prefix_path_for_game};
pub use profile::{load_profiles_from_dir, WineProfile};
pub use tuner::apply_profile;

/// Maps a game's crack type to its Wine profile name.
pub fn crack_type_to_profile_name(
    crack_type: Option<crate::library::game::CrackType>,
) -> &'static str {
    use crate::library::game::CrackType;
    match crack_type {
        Some(CrackType::OnlineFix) => "onlinefix",
        Some(CrackType::DODI) => "dodi",
        Some(CrackType::FitGirl) => "fitgirl",
        Some(CrackType::Scene) => "scene",
        Some(CrackType::GOGRip) => "gog",
        Some(CrackType::SteamRip) => "steam_generic",
        _ => "steam_generic",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::game::CrackType;

    #[test]
    fn test_crack_type_to_profile_name() {
        assert_eq!(
            crack_type_to_profile_name(Some(CrackType::OnlineFix)),
            "onlinefix"
        );
        assert_eq!(crack_type_to_profile_name(Some(CrackType::DODI)), "dodi");
        assert_eq!(
            crack_type_to_profile_name(Some(CrackType::FitGirl)),
            "fitgirl"
        );
        assert_eq!(crack_type_to_profile_name(Some(CrackType::Scene)), "scene");
        assert_eq!(crack_type_to_profile_name(Some(CrackType::GOGRip)), "gog");
        assert_eq!(
            crack_type_to_profile_name(Some(CrackType::SteamRip)),
            "steam_generic"
        );
        assert_eq!(
            crack_type_to_profile_name(Some(CrackType::Unknown)),
            "steam_generic"
        );
        assert_eq!(crack_type_to_profile_name(None), "steam_generic");
    }
}

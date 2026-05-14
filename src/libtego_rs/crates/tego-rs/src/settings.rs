// standard
use std::env;
use std::path::PathBuf;
use std::str::FromStr;

// extern
use anyhow::{bail, Result};
use rico_settings::v3::settings::Settings as SettingsV3;
use rico_settings::v4::settings::Settings as SettingsV4;

pub(crate) struct SettingsFile {
    settings_location: PathBuf,
    settings: SettingsV4,
}

impl SettingsFile {
    // Returns "%USERPROFILE%/AppData/Local/ricochet-refresh"
    #[cfg(target_os = "windows")]
    fn app_config_location() -> Result<PathBuf> {
        let userprofile = env::var("USERPROFILE")?;
        let mut config_location = PathBuf::from(userprofile);
        config_location.push("AppData/Local/ricochet-refresh");
        Ok(config_location)
    }

    /// Returns "~/.config/ricochet-refresh"
    #[cfg(target_os = "linux")]
    fn app_config_location() -> Result<PathBuf> {
        let home = env::var("HOME")?;
        let mut config_location = PathBuf::from(home);
        config_location.push(".config/ricochet-refresh");
        Ok(config_location)
    }

    /// Returns "~/Library/Preferences/ricochet-refresh"
    #[cfg(target_os = "macos")]
    fn app_config_location() -> Result<PathBuf> {
        let home = env::var("HOME")?;
        let mut config_location = PathBuf::from(home);
        config_location.push("Library/Preferences/ricochet-refresh");
        Ok(config_location)
    }

    /// Attempt to read settings from the default location
    /// If the v4 settings.json config file does not exist, we attempt to read
    /// the v3 ricochet.json config file.
    /// If v3 does not exist, we create a new default-initialised v4 settings.json file
    pub fn load_default() -> Result<SettingsFile> {
        let config_location = Self::app_config_location()?;
        let settings_location = config_location.join("settings.json");

        // load our default settings file if it exists
        let settings = if settings_location.exists() {
            let settings_json = std::fs::read_to_string(&settings_location)?;
            SettingsV4::from_str(settings_json.as_str())?
        } else {
            // attempt to load the legacy v3 settings if it exists
            let legacy_settings_location = config_location.join("ricochet.json");
            let settings = if legacy_settings_location.exists() {
                let legacy_settings_json = std::fs::read_to_string(&legacy_settings_location)?;
                let legacy_settings = SettingsV3::from_str(legacy_settings_json.as_str())?;
                SettingsV4::from(legacy_settings)
            // otherwise create a new default-initialised settings
            } else {
                std::fs::create_dir_all(&config_location)?;
                SettingsV4::default()
            };
            // write new settings to default location
            std::fs::write(&settings_location, settings.to_string())?;
            settings
        };

        Ok(SettingsFile {
            settings_location,
            settings,
        })
    }

    /// Attempt to red settings from a user-specified location
    /// If the file does not exist, a new config is created at the specified location
    /// Fails if the path to the file does not exist
    pub fn load_custom(settings_location: PathBuf) -> Result<SettingsFile> {
        let settings_location = std::path::absolute(settings_location.as_path())?;
        // load settings from src if it exits
        let settings = if settings_location.exists() {
            let settings_json = std::fs::read_to_string(&settings_location)?;
            SettingsV4::from_str(settings_json.as_str())?
        // otherewise write default settings to location
        } else {
            let settings = SettingsV4::default();
            std::fs::write(&settings_location, settings.to_string())?;
            settings
        };

        Ok(SettingsFile {
            settings_location,
            settings,
        })
    }

    pub fn flush(&self) -> Result<()> {
        std::fs::write(&self.settings_location, self.settings.to_string())?;
        Ok(())
    }
}

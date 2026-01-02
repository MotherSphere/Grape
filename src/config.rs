use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Dark
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextScale {
    Normal,
    Large,
    ExtraLarge,
}

impl Default for TextScale {
    fn default() -> Self {
        Self::Normal
    }
}

impl TextScale {
    pub fn scale(self) -> f32 {
        match self {
            Self::Normal => 1.0,
            Self::Large => 1.1,
            Self::ExtraLarge => 1.25,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Large => "Large",
            Self::ExtraLarge => "Très grand",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSettings {
    pub theme_mode: ThemeMode,
    pub text_scale: TextScale,
    pub default_volume: u8,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Dark,
            text_scale: TextScale::Normal,
            default_volume: 72,
        }
    }
}

impl UserSettings {
    pub fn normalized(mut self) -> Self {
        self.default_volume = self.default_volume.min(100);
        self
    }
}

fn config_root() -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".config").join("grape")
    } else if let Ok(profile) = env::var("USERPROFILE") {
        PathBuf::from(profile).join(".config").join("grape")
    } else {
        PathBuf::from(".")
    }
}

fn settings_path() -> PathBuf {
    config_root().join("preferences.json")
}

pub fn load_settings() -> UserSettings {
    let path = settings_path();
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return UserSettings::default();
        }
        Err(err) => {
            warn!(error = %err, path = %path.display(), "Failed to read preferences");
            return UserSettings::default();
        }
    };

    match serde_json::from_str::<UserSettings>(&contents) {
        Ok(settings) => settings.normalized(),
        Err(err) => {
            warn!(error = %err, path = %path.display(), "Failed to parse preferences");
            UserSettings::default()
        }
    }
}

pub fn save_settings(settings: &UserSettings) -> io::Result<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_string_pretty(settings)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    fs::write(path, payload)
}

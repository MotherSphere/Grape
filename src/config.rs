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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupScreen {
    Home,
    Library,
    Playlists,
    LastScreen,
}

impl Default for StartupScreen {
    fn default() -> Self {
        Self::Home
    }
}

impl StartupScreen {
    pub fn label(self) -> &'static str {
        match self {
            Self::Home => "Accueil",
            Self::Library => "Bibliothèque",
            Self::Playlists => "Playlists",
            Self::LastScreen => "Dernier écran",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloseBehavior {
    Quit,
    MinimizeToTray,
}

impl Default for CloseBehavior {
    fn default() -> Self {
        Self::Quit
    }
}

impl CloseBehavior {
    pub fn label(self) -> &'static str {
        match self {
            Self::Quit => "Quitter",
            Self::MinimizeToTray => "Réduire dans la barre",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceLanguage {
    System,
    French,
    English,
}

impl Default for InterfaceLanguage {
    fn default() -> Self {
        Self::System
    }
}

impl InterfaceLanguage {
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "Auto (système)",
            Self::French => "Français",
            Self::English => "Anglais",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeFormat {
    H24,
    H12,
}

impl Default for TimeFormat {
    fn default() -> Self {
        Self::H24
    }
}

impl TimeFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::H24 => "24h",
            Self::H12 => "12h",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateChannel {
    Stable,
    Beta,
}

impl Default for UpdateChannel {
    fn default() -> Self {
        Self::Stable
    }
}

impl UpdateChannel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::Beta => "Beta",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UserSettings {
    pub theme_mode: ThemeMode,
    pub text_scale: TextScale,
    pub default_volume: u8,
    pub launch_at_startup: bool,
    pub restore_last_session: bool,
    pub open_on: StartupScreen,
    pub close_behavior: CloseBehavior,
    pub interface_language: InterfaceLanguage,
    pub time_format: TimeFormat,
    pub auto_check_updates: bool,
    pub update_channel: UpdateChannel,
    pub auto_install_updates: bool,
    pub send_error_reports: bool,
    pub send_usage_stats: bool,
    pub library_folder: String,
    pub auto_scan_on_launch: bool,
    pub cache_path: String,
    pub notifications_enabled: bool,
    pub now_playing_notifications: bool,
    pub hardware_acceleration: bool,
    pub limit_cpu_during_playback: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Dark,
            text_scale: TextScale::Normal,
            default_volume: 72,
            launch_at_startup: false,
            restore_last_session: true,
            open_on: StartupScreen::Home,
            close_behavior: CloseBehavior::Quit,
            interface_language: InterfaceLanguage::System,
            time_format: TimeFormat::H24,
            auto_check_updates: true,
            update_channel: UpdateChannel::Stable,
            auto_install_updates: true,
            send_error_reports: false,
            send_usage_stats: false,
            library_folder: default_library_folder(),
            auto_scan_on_launch: true,
            cache_path: ".grape_cache".to_string(),
            notifications_enabled: true,
            now_playing_notifications: true,
            hardware_acceleration: true,
            limit_cpu_during_playback: false,
        }
    }
}

impl UserSettings {
    pub fn normalized(mut self) -> Self {
        self.default_volume = self.default_volume.min(100);
        if self.library_folder.trim().is_empty() {
            self.library_folder = default_library_folder();
        }
        if self.cache_path.trim().is_empty() {
            self.cache_path = ".grape_cache".to_string();
        }
        self
    }
}

fn default_library_folder() -> String {
    if let Ok(home) = env::var("HOME") {
        PathBuf::from(home)
            .join("Music")
            .to_string_lossy()
            .to_string()
    } else if let Ok(profile) = env::var("USERPROFILE") {
        PathBuf::from(profile)
            .join("Music")
            .to_string_lossy()
            .to_string()
    } else {
        ".".to_string()
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

fn history_path() -> PathBuf {
    config_root().join("history.json")
}

fn logs_dir() -> PathBuf {
    config_root().join("logs")
}

pub fn cache_dir(settings: &UserSettings) -> PathBuf {
    let path = PathBuf::from(&settings.cache_path);
    if path.is_absolute() {
        path
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(path)
    } else if let Ok(profile) = env::var("USERPROFILE") {
        PathBuf::from(profile).join(path)
    } else {
        path
    }
}

pub fn logs_path() -> PathBuf {
    logs_dir()
}

pub fn clear_history() -> io::Result<()> {
    let path = history_path();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn clear_cache(settings: &UserSettings) -> io::Result<()> {
    let path = cache_dir(settings);
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
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

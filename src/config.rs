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
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Dark
    }
}

impl ThemeMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Sombre",
            Self::Light => "Clair",
            Self::System => "Système",
        }
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

    pub fn slider_value(self) -> f32 {
        match self {
            Self::Normal => 0.0,
            Self::Large => 1.0,
            Self::ExtraLarge => 2.0,
        }
    }

    pub fn from_slider_value(value: f32) -> Self {
        match value.round() as i32 {
            0 => Self::Normal,
            1 => Self::Large,
            _ => Self::ExtraLarge,
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
pub enum AccentColor {
    Blue,
    Violet,
    Green,
    Amber,
}

impl Default for AccentColor {
    fn default() -> Self {
        Self::Blue
    }
}

impl AccentColor {
    pub fn label(self) -> &'static str {
        match self {
            Self::Blue => "Bleu",
            Self::Violet => "Violet",
            Self::Green => "Vert",
            Self::Amber => "Ambre",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceDensity {
    Compact,
    Comfort,
    Large,
}

impl Default for InterfaceDensity {
    fn default() -> Self {
        Self::Comfort
    }
}

impl InterfaceDensity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Comfort => "Confort",
            Self::Large => "Large",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioOutputDevice {
    System,
    UsbHeadset,
}

impl Default for AudioOutputDevice {
    fn default() -> Self {
        Self::System
    }
}

impl AudioOutputDevice {
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "Système",
            Self::UsbHeadset => "Casque USB",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingDeviceBehavior {
    SwitchToSystem,
    PausePlayback,
}

impl Default for MissingDeviceBehavior {
    fn default() -> Self {
        Self::SwitchToSystem
    }
}

impl MissingDeviceBehavior {
    pub fn label(self) -> &'static str {
        match self {
            Self::SwitchToSystem => "Basculer vers Système",
            Self::PausePlayback => "Mettre en pause",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeLevel {
    Quiet,
    Normal,
    Loud,
}

impl Default for VolumeLevel {
    fn default() -> Self {
        Self::Normal
    }
}

impl VolumeLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Quiet => "Quiet",
            Self::Normal => "Normal",
            Self::Loud => "Loud",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EqPreset {
    Flat,
    Bass,
    Treble,
    Vocal,
    Custom,
}

impl Default for EqPreset {
    fn default() -> Self {
        Self::Flat
    }
}

impl EqPreset {
    pub fn label(self) -> &'static str {
        match self {
            Self::Flat => "Flat",
            Self::Bass => "Bass",
            Self::Treble => "Treble",
            Self::Vocal => "Vocal",
            Self::Custom => "Custom…",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioStabilityMode {
    Auto,
    Stable,
    LowLatency,
}

impl Default for AudioStabilityMode {
    fn default() -> Self {
        Self::Auto
    }
}

impl AudioStabilityMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Stable => "Stable",
            Self::LowLatency => "Low-latency",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UserSettings {
    pub theme_mode: ThemeMode,
    pub follow_system_theme: bool,
    pub accent_color: AccentColor,
    pub accent_auto: bool,
    pub text_scale: TextScale,
    pub interface_density: InterfaceDensity,
    pub transparency_blur: bool,
    pub ui_animations: bool,
    pub default_volume: u8,
    pub output_device: AudioOutputDevice,
    pub missing_device_behavior: MissingDeviceBehavior,
    pub gapless_playback: bool,
    pub crossfade_seconds: u8,
    pub automix_enabled: bool,
    pub normalize_volume: bool,
    pub volume_level: VolumeLevel,
    pub eq_enabled: bool,
    pub eq_preset: EqPreset,
    pub audio_stability_mode: AudioStabilityMode,
    pub audio_debug_logs: bool,
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
            follow_system_theme: false,
            accent_color: AccentColor::default(),
            accent_auto: true,
            text_scale: TextScale::Normal,
            interface_density: InterfaceDensity::default(),
            transparency_blur: true,
            ui_animations: true,
            default_volume: 72,
            output_device: AudioOutputDevice::default(),
            missing_device_behavior: MissingDeviceBehavior::default(),
            gapless_playback: true,
            crossfade_seconds: 4,
            automix_enabled: false,
            normalize_volume: true,
            volume_level: VolumeLevel::default(),
            eq_enabled: false,
            eq_preset: EqPreset::default(),
            audio_stability_mode: AudioStabilityMode::default(),
            audio_debug_logs: false,
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
        self.crossfade_seconds = self.crossfade_seconds.min(12);
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

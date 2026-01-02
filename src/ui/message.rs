#![allow(dead_code)]

use crate::config::{
    AudioOutputDevice, AudioStabilityMode, CloseBehavior, EqPreset, InterfaceLanguage,
    MissingDeviceBehavior, StartupScreen, TextScale, ThemeMode, TimeFormat, UpdateChannel,
    VolumeLevel,
};
use crate::ui::state::{
    ActiveTab, Album, Artist, Folder, Genre, PreferencesSection, PreferencesTab, SortOption,
    Track,
};

#[derive(Debug, Clone, PartialEq)]
pub enum UiMessage {
    TabSelected(ActiveTab),
    SelectArtist(Artist),
    SelectAlbum(Album),
    SelectGenre(Genre),
    SelectFolder(Folder),
    SelectTrack(Track),
    Playback(PlaybackMessage),
    Search(SearchMessage),
    ToggleLogoMenu,
    OpenPlaylist,
    ClosePlaylist,
    ShowLibrary,
    OpenPreferences,
    ClosePreferences,
    PreferencesTabSelected(PreferencesTab),
    SetThemeMode(ThemeMode),
    SetTextScale(TextScale),
    SetDefaultVolume(u8),
    SetAudioOutputDevice(AudioOutputDevice),
    SetMissingDeviceBehavior(MissingDeviceBehavior),
    SetGaplessPlayback(bool),
    SetCrossfadeSeconds(u8),
    SetAutomixEnabled(bool),
    SetNormalizeVolume(bool),
    SetVolumeLevel(VolumeLevel),
    SetEqEnabled(bool),
    SetEqPreset(EqPreset),
    ResetEq,
    SetAudioStabilityMode(AudioStabilityMode),
    ResetAudioEngine,
    SetAudioDebugLogs(bool),
    SetLaunchAtStartup(bool),
    SetRestoreLastSession(bool),
    SetOpenOn(StartupScreen),
    SetCloseBehavior(CloseBehavior),
    SetInterfaceLanguage(InterfaceLanguage),
    SetTimeFormat(TimeFormat),
    SetAutoCheckUpdates(bool),
    SetUpdateChannel(UpdateChannel),
    SetAutoInstallUpdates(bool),
    SetSendErrorReports(bool),
    SetSendUsageStats(bool),
    LibraryFolderChanged(String),
    PickLibraryFolder,
    LibraryFolderPicked(Option<String>),
    SetAutoScanOnLaunch(bool),
    CachePathChanged(String),
    ClearCache,
    ClearHistory,
    SetNotificationsEnabled(bool),
    SetNowPlayingNotifications(bool),
    SetHardwareAcceleration(bool),
    SetLimitCpuDuringPlayback(bool),
    OpenLogsFolder,
    ReindexLibrary,
    ResetPreferences,
    TogglePreferencesSection(PreferencesSection),
    CloseMenu,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackMessage {
    TogglePlayPause,
    NextTrack,
    PreviousTrack,
    ToggleShuffle,
    CycleRepeat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchMessage {
    QueryChanged(String),
    SortChanged(SortOption),
}

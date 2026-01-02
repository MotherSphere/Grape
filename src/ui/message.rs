#![allow(dead_code)]

use crate::config::{
    CloseBehavior, InterfaceLanguage, StartupScreen, TextScale, ThemeMode, TimeFormat,
    UpdateChannel,
};
use crate::ui::state::{
    ActiveTab, Album, Artist, Folder, Genre, PreferencesTab, SortOption, Track,
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

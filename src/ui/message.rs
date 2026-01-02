#![allow(dead_code)]

use crate::config::{TextScale, ThemeMode};
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

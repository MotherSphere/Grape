#![allow(dead_code)]

use std::path::PathBuf;
use std::time::Duration;

use crate::config::UserSettings;
use crate::ui::message::{PlaybackMessage, SearchMessage, UiMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Artists,
    Genres,
    Albums,
    Folders,
}

impl Default for ActiveTab {
    fn default() -> Self {
        Self::Artists
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferencesTab {
    General,
    Appearance,
    Accessibility,
    Audio,
}

impl Default for PreferencesTab {
    fn default() -> Self {
        Self::General
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artist {
    pub id: usize,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Album {
    pub id: usize,
    pub title: String,
    pub artist: String,
    pub year: Option<u32>,
    pub cover_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Genre {
    pub id: usize,
    pub name: String,
    pub track_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    pub id: usize,
    pub name: String,
    pub track_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub id: usize,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub track_number: Option<u32>,
    pub duration: Duration,
    pub path: PathBuf,
    pub cover_path: Option<PathBuf>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SelectionState {
    pub selected_artist: Option<Artist>,
    pub selected_album: Option<Album>,
    pub selected_genre: Option<Genre>,
    pub selected_folder: Option<Folder>,
    pub selected_track: Option<Track>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

impl Default for RepeatMode {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PlaybackState {
    pub position: Duration,
    pub duration: Duration,
    pub is_playing: bool,
    pub shuffle: bool,
    pub repeat: RepeatMode,
}

impl PlaybackState {
    pub fn update(&mut self, message: PlaybackMessage) {
        match message {
            PlaybackMessage::ToggleShuffle => {
                self.shuffle = !self.shuffle;
            }
            PlaybackMessage::CycleRepeat => {
                self.repeat = match self.repeat {
                    RepeatMode::Off => RepeatMode::All,
                    RepeatMode::All => RepeatMode::One,
                    RepeatMode::One => RepeatMode::Off,
                };
            }
            PlaybackMessage::TogglePlayPause
            | PlaybackMessage::NextTrack
            | PlaybackMessage::PreviousTrack => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOption {
    Alphabetical,
    ByAlbum,
}

impl Default for SortOption {
    fn default() -> Self {
        Self::Alphabetical
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SearchState {
    pub query: String,
    pub sort: SortOption,
}

impl SearchState {
    pub fn update(&mut self, message: SearchMessage) {
        match message {
            SearchMessage::QueryChanged(query) => {
                self.query = query;
            }
            SearchMessage::SortChanged(sort) => {
                self.sort = sort;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiState {
    pub active_tab: ActiveTab,
    pub selection: SelectionState,
    pub playback: PlaybackState,
    pub search: SearchState,
    pub menu_open: bool,
    pub playlist_open: bool,
    pub preferences_open: bool,
    pub preferences_tab: PreferencesTab,
    pub settings: UserSettings,
}

impl UiState {
    pub fn new(settings: UserSettings) -> Self {
        Self {
            active_tab: ActiveTab::default(),
            selection: SelectionState::default(),
            playback: PlaybackState::default(),
            search: SearchState::default(),
            menu_open: false,
            playlist_open: false,
            preferences_open: false,
            preferences_tab: PreferencesTab::default(),
            settings,
        }
    }

    pub fn update(&mut self, message: UiMessage) {
        match message {
            UiMessage::TabSelected(tab) => {
                self.active_tab = tab;
                self.playlist_open = false;
                self.preferences_open = false;
            }
            UiMessage::SelectArtist(artist) => {
                self.selection.selected_artist = Some(artist);
                self.selection.selected_album = None;
                self.selection.selected_track = None;
            }
            UiMessage::SelectAlbum(album) => {
                self.selection.selected_album = Some(album);
                self.selection.selected_track = None;
            }
            UiMessage::SelectGenre(genre) => {
                self.selection.selected_genre = Some(genre);
                self.selection.selected_folder = None;
            }
            UiMessage::SelectFolder(folder) => {
                self.selection.selected_folder = Some(folder);
                self.selection.selected_genre = None;
            }
            UiMessage::SelectTrack(track) => {
                self.selection.selected_track = Some(track);
            }
            UiMessage::Playback(message) => {
                self.playback.update(message);
            }
            UiMessage::Search(message) => {
                self.search.update(message);
            }
            UiMessage::ToggleLogoMenu => {
                self.menu_open = !self.menu_open;
            }
            UiMessage::OpenPlaylist => {
                self.menu_open = false;
                self.playlist_open = true;
                self.preferences_open = false;
            }
            UiMessage::ClosePlaylist => {
                self.playlist_open = false;
            }
            UiMessage::ShowLibrary => {
                self.menu_open = false;
                self.playlist_open = false;
                self.preferences_open = false;
            }
            UiMessage::OpenPreferences => {
                self.menu_open = false;
                self.playlist_open = false;
                self.preferences_open = true;
            }
            UiMessage::ClosePreferences => {
                self.preferences_open = false;
            }
            UiMessage::PreferencesTabSelected(tab) => {
                self.preferences_tab = tab;
            }
            UiMessage::SetThemeMode(theme_mode) => {
                self.settings.theme_mode = theme_mode;
            }
            UiMessage::SetTextScale(scale) => {
                self.settings.text_scale = scale;
            }
            UiMessage::SetDefaultVolume(volume) => {
                self.settings.default_volume = volume.min(100);
            }
            UiMessage::CloseMenu => {
                self.menu_open = false;
            }
        }
    }
}

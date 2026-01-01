use std::time::Duration;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub id: usize,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub track_number: Option<u32>,
    pub duration: Duration,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SelectionState {
    pub selected_artist: Option<Artist>,
    pub selected_album: Option<Album>,
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
            PlaybackMessage::TogglePlayPause => {
                self.is_playing = !self.is_playing;
            }
            PlaybackMessage::NextTrack | PlaybackMessage::PreviousTrack => {
                self.position = Duration::ZERO;
                self.is_playing = true;
            }
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UiState {
    pub active_tab: ActiveTab,
    pub selection: SelectionState,
    pub playback: PlaybackState,
    pub search: SearchState,
}

impl UiState {
    pub fn update(&mut self, message: UiMessage) {
        match message {
            UiMessage::TabSelected(tab) => {
                self.active_tab = tab;
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
            UiMessage::SelectTrack(track) => {
                self.selection.selected_track = Some(track);
            }
            UiMessage::Playback(message) => {
                self.playback.update(message);
            }
            UiMessage::Search(message) => {
                self.search.update(message);
            }
        }
    }
}

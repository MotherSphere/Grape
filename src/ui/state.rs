use std::time::Duration;

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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UiState {
    pub active_tab: ActiveTab,
    pub selection: SelectionState,
    pub playback: PlaybackState,
    pub search: SearchState,
}

#![allow(dead_code)]

use crate::ui::state::{ActiveTab, Album, Artist, SortOption, Track};

#[derive(Debug, Clone, PartialEq)]
pub enum UiMessage {
    ToggleLogoMenu,
    OpenPlaylistWindow,
    CloseOverlays,
    Noop,
    TabSelected(ActiveTab),
    SelectArtist(Artist),
    SelectAlbum(Album),
    SelectTrack(Track),
    Playback(PlaybackMessage),
    Playlist(PlaylistMessage),
    Search(SearchMessage),
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

#[derive(Debug, Clone, PartialEq)]
pub enum PlaylistMessage {
    NameChanged(String),
    Create,
    AddTrack(Track),
    RemoveTrack(usize),
}

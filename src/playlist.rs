#![allow(dead_code)]

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::player::NowPlaying;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
    pub name: String,
    pub items: Vec<NowPlaying>,
}

impl Playlist {
    pub fn empty(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            items: Vec::new(),
        }
    }

    pub fn add(&mut self, item: NowPlaying) {
        self.items.push(item);
    }

    pub fn remove(&mut self, index: usize) -> Option<NowPlaying> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    pub fn reorder(&mut self, from: usize, to: usize) -> bool {
        if from >= self.items.len() || to >= self.items.len() {
            return false;
        }
        if from == to {
            return true;
        }
        let item = self.items.remove(from);
        self.items.insert(to, item);
        true
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn to_exchange(&self) -> PlaylistExchange {
        PlaylistExchange {
            version: PlaylistExchange::VERSION,
            name: self.name.clone(),
            items: self.items.iter().map(PlaylistItem::from).collect(),
        }
    }

    pub fn from_exchange(exchange: PlaylistExchange) -> Self {
        Self {
            name: exchange.name,
            items: exchange.items.into_iter().map(NowPlaying::from).collect(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.to_exchange())
    }

    pub fn from_json(payload: &str) -> Result<Self, serde_json::Error> {
        let exchange: PlaylistExchange = serde_json::from_str(payload)?;
        Ok(Self::from_exchange(exchange))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistManager {
    pub playlists: Vec<Playlist>,
    pub active_index: usize,
}

impl PlaylistManager {
    pub fn new_default() -> Self {
        Self {
            playlists: vec![Playlist::empty("Queue")],
            active_index: 0,
        }
    }

    pub fn active(&self) -> Option<&Playlist> {
        self.playlists.get(self.active_index)
    }

    pub fn add(&mut self, item: NowPlaying) {
        if let Some(playlist) = self.playlists.get_mut(self.active_index) {
            playlist.add(item);
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<NowPlaying> {
        self.playlists
            .get_mut(self.active_index)
            .and_then(|playlist| playlist.remove(index))
    }

    pub fn reorder(&mut self, from: usize, to: usize) -> bool {
        self.playlists
            .get_mut(self.active_index)
            .map_or(false, |playlist| playlist.reorder(from, to))
    }

    pub fn clear(&mut self) {
        if let Some(playlist) = self.playlists.get_mut(self.active_index) {
            playlist.clear();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaylistExchange {
    pub version: u32,
    pub name: String,
    pub items: Vec<PlaylistItem>,
}

impl PlaylistExchange {
    pub const VERSION: u32 = 1;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub artist: String,
    pub album: String,
    pub title: String,
    pub duration_secs: u32,
    pub path: String,
}

impl From<&NowPlaying> for PlaylistItem {
    fn from(item: &NowPlaying) -> Self {
        Self {
            artist: item.artist.clone(),
            album: item.album.clone(),
            title: item.title.clone(),
            duration_secs: item.duration_secs,
            path: item.path.to_string_lossy().into_owned(),
        }
    }
}

impl From<PlaylistItem> for NowPlaying {
    fn from(item: PlaylistItem) -> Self {
        Self {
            artist: item.artist,
            album: item.album,
            title: item.title,
            duration_secs: item.duration_secs,
            path: PathBuf::from(item.path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_track(index: u32) -> NowPlaying {
        NowPlaying {
            artist: format!("Artist {index}"),
            album: format!("Album {index}"),
            title: format!("Track {index}"),
            duration_secs: 180 + index,
            path: PathBuf::from(format!("/music/track_{index}.mp3")),
        }
    }

    #[test]
    fn add_remove_items() {
        let mut playlist = Playlist::empty("Favorites");
        let first = sample_track(1);
        let second = sample_track(2);

        playlist.add(first.clone());
        playlist.add(second.clone());

        assert_eq!(playlist.items.len(), 2);
        assert_eq!(playlist.remove(0), Some(first));
        assert_eq!(playlist.items.len(), 1);
        assert_eq!(playlist.remove(10), None);
        assert_eq!(playlist.items, vec![second]);
    }

    #[test]
    fn reorder_items() {
        let mut playlist = Playlist::empty("Mix");
        let first = sample_track(1);
        let second = sample_track(2);
        let third = sample_track(3);

        playlist.add(first.clone());
        playlist.add(second.clone());
        playlist.add(third.clone());

        assert!(playlist.reorder(0, 2));
        assert_eq!(
            playlist.items,
            vec![second.clone(), third.clone(), first.clone()]
        );
        assert!(playlist.reorder(1, 1));
        assert!(!playlist.reorder(10, 0));
    }

    #[test]
    fn exchange_and_json_roundtrip() {
        let mut playlist = Playlist::empty("Roadtrip");
        playlist.add(sample_track(1));
        playlist.add(sample_track(2));

        let exchange = playlist.to_exchange();
        assert_eq!(exchange.version, PlaylistExchange::VERSION);

        let json = playlist.to_json().expect("serialize playlist");
        let parsed = Playlist::from_json(&json).expect("deserialize playlist");

        assert_eq!(playlist, parsed);
    }
}

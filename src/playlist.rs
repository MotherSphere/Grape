use crate::player::NowPlaying;

#[derive(Debug, Clone)]
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
}

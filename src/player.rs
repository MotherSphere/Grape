#[derive(Debug, Clone)]
pub struct NowPlaying {
    pub artist: String,
    pub album: String,
    pub title: String,
    pub duration_secs: u32,
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub now_playing: Option<NowPlaying>,
    pub is_playing: bool,
    pub position_secs: u32,
}

impl PlayerState {
    pub fn placeholder() -> Self {
        Self {
            now_playing: None,
            is_playing: false,
            position_secs: 0,
        }
    }
}

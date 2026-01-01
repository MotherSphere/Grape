#[derive(Debug, Clone)]
pub struct Catalog {
    pub artists: Vec<Artist>,
}

#[derive(Debug, Clone)]
pub struct Artist {
    pub name: String,
    pub albums: Vec<Album>,
}

#[derive(Debug, Clone)]
pub struct Album {
    pub title: String,
    pub year: u16,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub number: u8,
    pub title: String,
    pub duration_secs: u32,
}

impl Catalog {
    pub fn first_track(&self) -> Option<(&Artist, &Album, &Track)> {
        let artist = self.artists.first()?;
        let album = artist.albums.first()?;
        let track = album.tracks.first()?;
        Some((artist, album, track))
    }
}

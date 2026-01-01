use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
    pub path: PathBuf,
}

impl Catalog {
    pub fn empty() -> Self {
        Self { artists: Vec::new() }
    }

    pub fn first_track(&self) -> Option<(&Artist, &Album, &Track)> {
        let artist = self.artists.first()?;
        let album = artist.albums.first()?;
        let track = album.tracks.first()?;
        Some((artist, album, track))
    }
}

pub fn scan_library(root: impl AsRef<Path>) -> io::Result<Catalog> {
    let root = root.as_ref();
    let mut artists = Vec::new();

    if !root.exists() {
        return Ok(Catalog::empty());
    }

    for artist_entry in read_sorted_dirs(root)? {
        let artist_name = artist_entry
            .file_name()
            .to_string_lossy()
            .trim()
            .to_string();
        let mut albums = Vec::new();

        for album_entry in read_sorted_dirs(&artist_entry.path())? {
            let (year, title) = parse_album_folder(&album_entry.file_name().to_string_lossy());
            let tracks = scan_tracks(&album_entry.path())?;

            if !tracks.is_empty() {
                albums.push(Album { title, year, tracks });
            }
        }

        if !albums.is_empty() {
            artists.push(Artist {
                name: artist_name,
                albums,
            });
        }
    }

    Ok(Catalog { artists })
}

fn scan_tracks(dir: &Path) -> io::Result<Vec<Track>> {
    let mut tracks = Vec::new();
    let mut index = 1u8;

    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .collect();

    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        if !is_audio_file(&path) {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let (number, title) = parse_track_filename(stem);
        let track_number = number.unwrap_or_else(|| {
            let current = index;
            index = index.saturating_add(1);
            current
        });

        tracks.push(Track {
            number: track_number,
            title,
            duration_secs: 0,
            path,
        });
    }

    tracks.sort_by_key(|track| track.number);
    Ok(tracks)
}

fn read_sorted_dirs(root: &Path) -> io::Result<Vec<fs::DirEntry>> {
    let mut entries: Vec<_> = fs::read_dir(root)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

fn is_audio_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()).map(str::to_lowercase),
        Some(ext) if matches!(ext.as_str(), "mp3" | "flac" | "wav" | "ogg" | "m4a")
    )
}

fn parse_album_folder(name: &str) -> (u16, String) {
    let trimmed = name.trim();
    let mut year_end = 0usize;
    for (idx, ch) in trimmed.char_indices() {
        if ch.is_ascii_digit() {
            year_end = idx + ch.len_utf8();
        } else {
            break;
        }
    }

    if year_end >= 4 {
        let year_part = &trimmed[..year_end];
        if let Ok(year) = year_part.parse::<u16>() {
            let title = trimmed[year_end..]
                .trim_start_matches(|c: char| c == '-' || c == '_' || c.is_whitespace())
                .trim();
            let title = if title.is_empty() { trimmed } else { title };
            return (year, title.to_string());
        }
    }

    (0, trimmed.to_string())
}

fn parse_track_filename(name: &str) -> (Option<u8>, String) {
    let trimmed = name.trim();
    let mut number_end = 0usize;
    for (idx, ch) in trimmed.char_indices() {
        if ch.is_ascii_digit() {
            number_end = idx + ch.len_utf8();
        } else {
            break;
        }
    }

    if number_end > 0 {
        let number_part = &trimmed[..number_end];
        let title = trimmed[number_end..]
            .trim_start_matches(|c: char| c == '-' || c == '_' || c.is_whitespace())
            .trim();
        let title = if title.is_empty() { trimmed } else { title };
        return (number_part.parse::<u8>().ok(), title.to_string());
    }

    (None, trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn parse_track_filename_extracts_number_and_title() {
        let (number, title) = parse_track_filename("01 - Lumiere");
        assert_eq!(number, Some(1));
        assert_eq!(title, "Lumiere");

        let (number, title) = parse_track_filename("7_Titre");
        assert_eq!(number, Some(7));
        assert_eq!(title, "Titre");

        let (number, title) = parse_track_filename("Sans numero");
        assert_eq!(number, None);
        assert_eq!(title, "Sans numero");
    }

    #[test]
    fn scan_library_builds_catalog_from_folders() {
        let dir = tempdir().expect("tempdir");
        let artist_dir = dir.path().join("Artiste");
        let album_dir = artist_dir.join("2022 - Album");
        fs::create_dir_all(&album_dir).expect("create album dirs");

        File::create(album_dir.join("01 - Intro.mp3")).expect("create track");
        File::create(album_dir.join("02 - Suite.flac")).expect("create track");
        File::create(album_dir.join("notes.txt")).expect("create note");

        let catalog = scan_library(dir.path()).expect("scan library");
        assert_eq!(catalog.artists.len(), 1);
        let artist = &catalog.artists[0];
        assert_eq!(artist.name, "Artiste");
        assert_eq!(artist.albums.len(), 1);
        let album = &artist.albums[0];
        assert_eq!(album.year, 2022);
        assert_eq!(album.title, "Album");
        assert_eq!(album.tracks.len(), 2);
        assert_eq!(album.tracks[0].title, "Intro");
        assert_eq!(album.tracks[1].number, 2);
    }
}

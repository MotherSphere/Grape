use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::library::Album;

const CACHE_DIRNAME: &str = ".grape_cache";
const INDEX_FILENAME: &str = "index.json";
const FOLDERS_DIRNAME: &str = "folders";
const COVER_DIRNAME: &str = "covers";
const METADATA_DIRNAME: &str = "metadata";
const CACHE_VERSION: u32 = 4;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CacheIndex {
    version: u32,
    #[serde(default)]
    tracks: HashMap<String, TrackEntry>,
    #[serde(skip)]
    legacy_version: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FolderEntry {
    #[serde(default)]
    tracks: HashMap<String, TrackEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FolderCacheFile {
    album: Album,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct TrackEntry {
    modified_secs: u64,
    hash: u64,
}

pub struct CachedAlbum {
    pub album: Album,
}

impl CacheIndex {
    pub fn track_entries(&self) -> &HashMap<String, TrackEntry> {
        &self.tracks
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LegacyCacheIndex {
    version: u32,
    #[serde(default)]
    entries: HashMap<String, FolderEntry>,
}

pub fn load_index(root: &Path) -> io::Result<CacheIndex> {
    if !root.exists() {
        return Ok(CacheIndex::default());
    }

    let index_path = index_path(root);
    if !index_path.exists() {
        return Ok(CacheIndex::default());
    }

    let contents = fs::read_to_string(&index_path)?;
    let value: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    if value.get("entries").is_some() {
        let legacy: LegacyCacheIndex = serde_json::from_value(value)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        let mut tracks = HashMap::new();
        for entry in legacy.entries.values() {
            for (key, track_entry) in &entry.tracks {
                tracks.insert(key.clone(), *track_entry);
            }
        }
        return Ok(CacheIndex {
            version: CACHE_VERSION,
            tracks,
            legacy_version: Some(legacy.version),
        });
    }

    let mut index: CacheIndex = serde_json::from_value(value)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    if index.version != CACHE_VERSION {
        index.legacy_version = Some(index.version);
        index.version = CACHE_VERSION;
    }

    Ok(index)
}

pub fn load_album(
    root: &Path,
    album_path: &Path,
) -> io::Result<Option<CachedAlbum>> {
    let key = album_key(root, album_path)?;
    let cache_path = folder_cache_path(root, &key);
    if !cache_path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&cache_path)?;
    let mut cache_file: FolderCacheFile = serde_json::from_str(&contents)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    cache_file.album.path = album_path.to_path_buf();
    Ok(Some(CachedAlbum {
        album: cache_file.album,
    }))
}

pub fn store_album(
    root: &Path,
    index: &mut CacheIndex,
    album_path: &Path,
    album: &Album,
) -> io::Result<String> {
    let key = album_key(root, album_path)?;
    let cache_dir = root.join(CACHE_DIRNAME).join(FOLDERS_DIRNAME);
    fs::create_dir_all(&cache_dir)?;

    let cache_file = FolderCacheFile {
        album: album.clone(),
    };

    let contents = serde_json::to_string_pretty(&cache_file)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(folder_cache_path(root, &key), contents)?;

    let track_entries = build_track_entries(root, album);
    for (track_key, track_entry) in track_entries {
        index.tracks.insert(track_key, track_entry);
    }

    Ok(key)
}

pub fn finalize(
    root: &Path,
    index: &mut CacheIndex,
    used_keys: &HashSet<String>,
    used_track_keys: &HashSet<String>,
) -> io::Result<()> {
    if !root.exists() {
        return Ok(());
    }

    index.version = CACHE_VERSION;
    let folders_dir = root.join(CACHE_DIRNAME).join(FOLDERS_DIRNAME);
    if folders_dir.exists() {
        if let Ok(read_dir) = fs::read_dir(&folders_dir) {
            for entry_result in read_dir {
                let Ok(entry) = entry_result else {
                    continue;
                };
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }
                let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
                    continue;
                };
                if !used_keys.contains(stem) {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }

    let removed_tracks: Vec<String> = index
        .tracks
        .keys()
        .filter(|key| !used_track_keys.contains(*key))
        .cloned()
        .collect();

    for key in removed_tracks {
        index.tracks.remove(&key);
    }

    let contents = serde_json::to_string_pretty(index)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::create_dir_all(root.join(CACHE_DIRNAME))?;
    fs::write(index_path(root), contents)
}

pub fn ensure_cover_cache_dir(root: &Path) -> io::Result<PathBuf> {
    let dir = root.join(CACHE_DIRNAME).join(COVER_DIRNAME);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn ensure_metadata_cache_dir(root: &Path) -> io::Result<PathBuf> {
    let dir = root.join(CACHE_DIRNAME).join(METADATA_DIRNAME);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn index_path(root: &Path) -> PathBuf {
    root.join(CACHE_DIRNAME).join(INDEX_FILENAME)
}

fn folder_cache_path(root: &Path, key: &str) -> PathBuf {
    root.join(CACHE_DIRNAME)
        .join(FOLDERS_DIRNAME)
        .join(format!("{key}.json"))
}

fn album_key(root: &Path, album_path: &Path) -> io::Result<String> {
    let relative = relative_path(root, album_path);
    Ok(hash_key(&relative))
}

fn hash_key(value: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn track_key(root: &Path, track_path: &Path) -> String {
    relative_path(root, track_path)
}

pub fn track_signature(path: &Path) -> io::Result<TrackEntry> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
    let duration = modified.duration_since(UNIX_EPOCH).unwrap_or_default();
    let modified_secs = duration.as_secs();
    let file_len = metadata.len();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    file_len.hash(&mut hasher);
    modified_secs.hash(&mut hasher);
    Ok(TrackEntry {
        modified_secs,
        hash: hasher.finish(),
    })
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn build_track_entries(root: &Path, album: &Album) -> HashMap<String, TrackEntry> {
    let mut entries = HashMap::new();
    for track in &album.tracks {
        if let Ok(signature) = track_signature(&track.path) {
            let key = track_key(root, &track.path);
            entries.insert(key, signature);
        }
    }
    entries
}

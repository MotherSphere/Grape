use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::library::Album;

const CACHE_DIRNAME: &str = ".grape_cache";
const INDEX_FILENAME: &str = "index.json";
const FOLDERS_DIRNAME: &str = "folders";
const COVER_DIRNAME: &str = "covers";
const CACHE_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CacheIndex {
    version: u32,
    entries: HashMap<String, FolderEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FolderEntry {
    modified_secs: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct FolderCacheFile {
    album: Album,
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
    let mut index: CacheIndex = serde_json::from_str(&contents)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    if index.version != CACHE_VERSION {
        index = CacheIndex::default();
    }

    Ok(index)
}

pub fn load_album(root: &Path, index: &CacheIndex, album_path: &Path) -> io::Result<Option<Album>> {
    let key = album_key(root, album_path)?;
    let Some(entry) = index.entries.get(&key) else {
        return Ok(None);
    };

    let current_modified = folder_modified_secs(album_path)?;
    if current_modified != entry.modified_secs {
        return Ok(None);
    }

    let cache_path = folder_cache_path(root, &key);
    if !cache_path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&cache_path)?;
    let mut cache_file: FolderCacheFile = serde_json::from_str(&contents)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    cache_file.album.path = album_path.to_path_buf();
    Ok(Some(cache_file.album))
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

    let modified_secs = folder_modified_secs(album_path)?;
    index
        .entries
        .insert(key.clone(), FolderEntry { modified_secs });

    Ok(key)
}

pub fn finalize(
    root: &Path,
    index: &mut CacheIndex,
    used_keys: &HashSet<String>,
) -> io::Result<()> {
    if !root.exists() {
        return Ok(());
    }

    index.version = CACHE_VERSION;

    let removed_keys: Vec<String> = index
        .entries
        .keys()
        .filter(|key| !used_keys.contains(*key))
        .cloned()
        .collect();

    for key in removed_keys {
        index.entries.remove(&key);
        let cache_path = folder_cache_path(root, &key);
        if cache_path.exists() {
            let _ = fs::remove_file(cache_path);
        }
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

fn index_path(root: &Path) -> PathBuf {
    root.join(CACHE_DIRNAME).join(INDEX_FILENAME)
}

fn folder_cache_path(root: &Path, key: &str) -> PathBuf {
    root.join(CACHE_DIRNAME)
        .join(FOLDERS_DIRNAME)
        .join(format!("{key}.json"))
}

fn album_key(root: &Path, album_path: &Path) -> io::Result<String> {
    let relative = album_path
        .strip_prefix(root)
        .unwrap_or(album_path)
        .to_string_lossy()
        .replace('\\', "/");
    Ok(hash_key(&relative))
}

fn hash_key(value: &str) -> String {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn folder_modified_secs(path: &Path) -> io::Result<u64> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
    let duration = modified.duration_since(UNIX_EPOCH).unwrap_or_default();
    Ok(duration.as_secs())
}

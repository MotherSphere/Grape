use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::library::Catalog;

const CACHE_FILENAME: &str = ".grape_cache.json";

#[derive(Debug, Serialize, Deserialize)]
struct CacheFile {
    root_modified_secs: u64,
    catalog: Catalog,
}

pub fn load(root: &Path) -> io::Result<Option<Catalog>> {
    if !root.exists() {
        return Ok(None);
    }

    let cache_path = cache_path(root);
    if !cache_path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&cache_path)?;
    let cache: CacheFile = serde_json::from_str(&contents)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    let current_modified = root_modified_secs(root)?;

    if cache.root_modified_secs != current_modified {
        return Ok(None);
    }

    Ok(Some(cache.catalog))
}

pub fn store(root: &Path, catalog: &Catalog) -> io::Result<()> {
    if !root.exists() {
        return Ok(());
    }

    let cache = CacheFile {
        root_modified_secs: root_modified_secs(root)?,
        catalog: catalog.clone(),
    };

    let contents = serde_json::to_string_pretty(&cache)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(cache_path(root), contents)
}

fn cache_path(root: &Path) -> PathBuf {
    root.join(CACHE_FILENAME)
}

fn root_modified_secs(root: &Path) -> io::Result<u64> {
    let metadata = fs::metadata(root)?;
    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
    let duration = modified.duration_since(UNIX_EPOCH).unwrap_or_default();
    Ok(duration.as_secs())
}

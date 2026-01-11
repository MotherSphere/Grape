use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::config::UserSettings;
use crate::library::{CoverArt, cache};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OnlineMetadata {
    pub genre: Option<String>,
    pub year: Option<u16>,
    #[serde(default)]
    pub cover: Option<CoverArt>,
    #[serde(default)]
    pub cover_checked: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserMetadataOverride {
    pub genre: Option<String>,
    pub year: Option<u16>,
    #[serde(default)]
    pub genre_overridden: bool,
    #[serde(default)]
    pub year_overridden: bool,
    #[serde(default)]
    pub edited_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedOnlineMetadata {
    fetched_at: u64,
    metadata: OnlineMetadata,
    #[serde(default)]
    user_override: Option<UserMetadataOverride>,
}

pub fn load_user_metadata_override(
    root: &Path,
    artist: &str,
    album: &str,
) -> io::Result<Option<UserMetadataOverride>> {
    let cache_dir = cache::ensure_metadata_cache_dir(root)?;
    let cache_key = metadata_cache_key(artist, album);
    let cache_path = cache_dir.join(format!("{cache_key}.json"));
    if !cache_path.exists() {
        return Ok(None);
    }
    Ok(load_cached_metadata(&cache_path).and_then(|entry| entry.user_override))
}

pub fn store_user_metadata_override(
    root: &Path,
    artist: &str,
    album: &str,
    mut metadata_override: UserMetadataOverride,
) -> io::Result<()> {
    let cache_dir = cache::ensure_metadata_cache_dir(root)?;
    let cache_key = metadata_cache_key(artist, album);
    let cache_path = cache_dir.join(format!("{cache_key}.json"));
    let existing = if cache_path.exists() {
        load_cached_metadata(&cache_path)
    } else {
        None
    };
    metadata_override.edited_at = current_epoch_secs();
    let payload = CachedOnlineMetadata {
        fetched_at: existing.as_ref().map(|entry| entry.fetched_at).unwrap_or(0),
        metadata: existing
            .as_ref()
            .map(|entry| entry.metadata.clone())
            .unwrap_or_default(),
        user_override: Some(metadata_override),
    };
    write_metadata_cache(&cache_path, &payload)
}

pub fn fetch_album_metadata(
    root: &Path,
    settings: &UserSettings,
    artist: &str,
    album: &str,
    force_refresh: bool,
) -> io::Result<Option<OnlineMetadata>> {
    let api_key = settings.metadata_api_key.trim();
    if api_key.is_empty() {
        return Ok(None);
    }

    let cache_key = metadata_cache_key(artist, album);
    let cache_dir = cache::ensure_metadata_cache_dir(root)?;
    let cache_path = cache_dir.join(format!("{cache_key}.json"));
    let ttl_secs = u64::from(settings.metadata_cache_ttl_hours).saturating_mul(3600);
    let now_secs = current_epoch_secs();

    let cached = if cache_path.exists() {
        load_cached_metadata(&cache_path)
    } else {
        None
    };

    if !force_refresh {
        if let Some(entry) = &cached {
            if ttl_secs > 0 && now_secs.saturating_sub(entry.fetched_at) < ttl_secs {
                let cover_ready = entry
                    .metadata
                    .cover
                    .as_ref()
                    .map(|cover| cover.cached_path.exists())
                    .unwrap_or(false);
                if (entry.metadata.cover.is_some() && cover_ready) || entry.metadata.cover_checked {
                    return Ok(Some(entry.metadata.clone()));
                }
            }
        }
    }

    let base_metadata = cached
        .as_ref()
        .map(|entry| entry.metadata.clone())
        .unwrap_or_default();
    let metadata = match enrich_with_lastfm_metadata(
        root,
        base_metadata,
        api_key,
        artist,
        album,
        force_refresh,
    ) {
        Ok(metadata) => metadata,
        Err(error) => {
            warn!(error = %error, "Failed to fetch online metadata");
            return Ok(cached.map(|entry| entry.metadata));
        }
    };

    let payload = CachedOnlineMetadata {
        fetched_at: now_secs,
        metadata: metadata.clone(),
        user_override: cached.and_then(|entry| entry.user_override),
    };

    if let Err(error) = write_metadata_cache(&cache_path, &payload) {
        warn!(error = %error, "Failed to write online metadata cache");
    }

    Ok(Some(metadata))
}

fn enrich_with_lastfm_metadata(
    root: &Path,
    mut metadata: OnlineMetadata,
    api_key: &str,
    artist: &str,
    album: &str,
    force_refresh: bool,
) -> Result<OnlineMetadata, reqwest::Error> {
    let lastfm_metadata = fetch_lastfm_metadata(api_key, artist, album)?;
    if metadata.genre.is_none() {
        metadata.genre = lastfm_metadata.genre.clone();
    }
    if metadata.year.is_none() {
        metadata.year = lastfm_metadata.year;
    }
    if should_update_cover(&metadata, force_refresh) {
        if let Some(url) = lastfm_metadata.cover_url.as_deref() {
            if let Ok(cover) =
                download_cover_art(root, &metadata_cache_key(artist, album), url, force_refresh)
            {
                metadata.cover = cover;
            }
        }
    }
    metadata.cover_checked = true;
    Ok(metadata)
}

#[derive(Debug, Clone)]
struct LastFmMetadata {
    genre: Option<String>,
    year: Option<u16>,
    cover_url: Option<String>,
}

fn fetch_lastfm_metadata(
    api_key: &str,
    artist: &str,
    album: &str,
) -> Result<LastFmMetadata, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://ws.audioscrobbler.com/2.0/")
        .query(&[
            ("method", "album.getInfo"),
            ("api_key", api_key),
            ("artist", artist),
            ("album", album),
            ("format", "json"),
        ])
        .send()?
        .error_for_status()?;

    let payload: serde_json::Value = response.json()?;
    let genre = extract_genre(&payload);
    let year = extract_year(&payload);
    let cover_url = extract_cover_url(&payload);

    Ok(LastFmMetadata {
        genre,
        year,
        cover_url,
    })
}

fn extract_genre(payload: &serde_json::Value) -> Option<String> {
    let tag_value = payload
        .pointer("/album/toptags/tag")
        .or_else(|| payload.pointer("/album/tags/tag"));

    match tag_value {
        Some(value) if value.is_array() => value
            .as_array()
            .and_then(|tags| tags.iter().find_map(|tag| tag.get("name")))
            .and_then(|name| name.as_str())
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty()),
        Some(value) if value.is_object() => value
            .get("name")
            .and_then(|name| name.as_str())
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty()),
        _ => None,
    }
}

fn extract_year(payload: &serde_json::Value) -> Option<u16> {
    let release = payload
        .pointer("/album/releasedate")
        .and_then(|value| value.as_str())
        .or_else(|| {
            payload
                .pointer("/album/wiki/published")
                .and_then(|value| value.as_str())
        });
    release.and_then(parse_year)
}

fn extract_cover_url(payload: &serde_json::Value) -> Option<String> {
    let images = payload.pointer("/album/image")?;
    let list = images.as_array()?;
    let mut fallback = None;
    for image in list {
        let url = image.get("#text").and_then(|value| value.as_str());
        let url = url.map(str::trim).filter(|url| !url.is_empty());
        let size = image.get("size").and_then(|value| value.as_str());
        if let Some(url) = url {
            match size {
                Some("mega") | Some("extralarge") => return Some(url.to_string()),
                _ => {
                    if fallback.is_none() {
                        fallback = Some(url.to_string());
                    }
                }
            }
        }
    }
    fallback
}

fn parse_year(value: &str) -> Option<u16> {
    let mut digits = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            if digits.len() == 4 {
                if let Ok(year) = digits.parse::<u16>() {
                    if year > 0 {
                        return Some(year);
                    }
                }
                digits.clear();
            }
        } else {
            digits.clear();
        }
    }
    None
}

fn should_update_cover(metadata: &OnlineMetadata, force_refresh: bool) -> bool {
    if force_refresh {
        return true;
    }
    match metadata.cover.as_ref() {
        Some(cover) => !cover.cached_path.exists(),
        None => !metadata.cover_checked,
    }
}

fn download_cover_art(
    root: &Path,
    cache_key: &str,
    url: &str,
    force_refresh: bool,
) -> io::Result<Option<CoverArt>> {
    let cache_dir = cache::ensure_cover_cache_dir(root)?;
    let extension = cover_extension_from_url(url);
    let cached_path = cache_dir.join(format!("online-{cache_key}.{extension}"));
    if cached_path.exists() && !force_refresh {
        return Ok(Some(CoverArt {
            source_path: cached_path.clone(),
            cached_path,
            modified_secs: current_epoch_secs(),
        }));
    }
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url)
        .send()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
        .error_for_status()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    let bytes = response
        .bytes()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    fs::write(&cached_path, &bytes)?;
    Ok(Some(CoverArt {
        source_path: cached_path.clone(),
        cached_path,
        modified_secs: current_epoch_secs(),
    }))
}

fn cover_extension_from_url(url: &str) -> &str {
    let trimmed = url.split('?').next().unwrap_or(url);
    std::path::Path::new(trimmed)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("jpg")
}

fn metadata_cache_key(artist: &str, album: &str) -> String {
    let input = format!(
        "{}::{}",
        artist.trim().to_lowercase(),
        album.trim().to_lowercase()
    );
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::Hash;
    use std::hash::Hasher;
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn load_cached_metadata(path: &Path) -> Option<CachedOnlineMetadata> {
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<CachedOnlineMetadata>(&contents) {
            Ok(entry) => Some(entry),
            Err(error) => {
                warn!(error = %error, "Failed to parse cached online metadata");
                None
            }
        },
        Err(error) => {
            warn!(error = %error, "Failed to read cached online metadata");
            None
        }
    }
}

fn write_metadata_cache(path: &Path, payload: &CachedOnlineMetadata) -> io::Result<()> {
    let contents = serde_json::to_string_pretty(payload)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    let temp_path = path.with_extension("json.tmp");
    fs::write(&temp_path, contents)?;
    fs::rename(&temp_path, path)
}

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

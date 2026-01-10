use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::config::UserSettings;
use crate::library::cache;
use crate::library::metadata;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnlineMetadata {
    pub genre: Option<String>,
    pub year: Option<u16>,
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
                return Ok(Some(entry.metadata.clone()));
            }
        }
    }

    let base_metadata = cached
        .as_ref()
        .map(|entry| entry.metadata.clone())
        .unwrap_or_default();
    let (metadata, refreshed) =
        enrich_with_online_metadata(base_metadata, api_key, artist, album);
    if !refreshed {
        return Ok(cached.map(|entry| entry.metadata));
    }

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

fn enrich_with_online_metadata(
    mut metadata: OnlineMetadata,
    api_key: &str,
    artist: &str,
    album: &str,
) -> (OnlineMetadata, bool) {
    let mut refreshed = false;

    if !api_key.is_empty() {
        match fetch_lastfm_metadata(api_key, artist, album) {
            Ok(lastfm_metadata) => {
                refreshed |= lastfm_metadata.genre.is_some() || lastfm_metadata.year.is_some();
                metadata = merge_online_metadata(metadata, lastfm_metadata);
            }
            Err(error) => {
                warn!(error = %error, "Failed to fetch Last.fm metadata");
            }
        }
    }

    match fetch_musicbrainz_metadata(artist, album) {
        Ok(musicbrainz_metadata) => {
            refreshed |= musicbrainz_metadata.genre.is_some() || musicbrainz_metadata.year.is_some();
            metadata = merge_online_metadata(metadata, musicbrainz_metadata);
        }
        Err(error) => {
            warn!(error = %error, "Failed to fetch MusicBrainz metadata");
        }
    }

    match fetch_itunes_metadata(artist, album) {
        Ok(itunes_metadata) => {
            refreshed |= itunes_metadata.genre.is_some() || itunes_metadata.year.is_some();
            metadata = merge_online_metadata(metadata, itunes_metadata);
        }
        Err(error) => {
            warn!(error = %error, "Failed to fetch iTunes metadata");
        }
    }

    (metadata, refreshed)
}

fn fetch_lastfm_metadata(
    api_key: &str,
    artist: &str,
    album: &str,
) -> Result<OnlineMetadata, reqwest::Error> {
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
    let genre = extract_genre(&payload).and_then(|value| metadata::merge_genres(Some(&value), None));
    let year = extract_year(&payload);

    Ok(OnlineMetadata { genre, year })
}

fn fetch_musicbrainz_metadata(artist: &str, album: &str) -> Result<OnlineMetadata, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let query = format!("artist:\"{}\" AND releasegroup:\"{}\"", artist, album);
    let response = client
        .get("https://musicbrainz.org/ws/2/release-group/")
        .header("User-Agent", "Grape/0.1 (metadata)")
        .query(&[("query", query), ("fmt", "json".to_string()), ("limit", "1".to_string())])
        .send()?
        .error_for_status()?;

    let payload: serde_json::Value = response.json()?;
    let release_group = payload.pointer("/release-groups/0");
    let genre = release_group
        .and_then(|value| value.get("tags"))
        .and_then(|tags| tags.as_array())
        .and_then(|tags| tags.iter().find_map(|tag| tag.get("name")))
        .and_then(|name| name.as_str())
        .and_then(|name| metadata::merge_genres(Some(name), None));
    let year = release_group
        .and_then(|value| value.get("first-release-date"))
        .and_then(|value| value.as_str())
        .and_then(parse_year);

    Ok(OnlineMetadata { genre, year })
}

fn fetch_itunes_metadata(artist: &str, album: &str) -> Result<OnlineMetadata, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let term = format!("{} {}", artist, album);
    let response = client
        .get("https://itunes.apple.com/search")
        .query(&[("term", term), ("entity", "album".to_string()), ("limit", "1".to_string())])
        .send()?
        .error_for_status()?;

    let payload: serde_json::Value = response.json()?;
    let result = payload.pointer("/results/0");
    let genre = result
        .and_then(|value| value.get("primaryGenreName"))
        .and_then(|value| value.as_str())
        .and_then(|name| metadata::merge_genres(Some(name), None));
    let year = result
        .and_then(|value| value.get("releaseDate"))
        .and_then(|value| value.as_str())
        .and_then(parse_year);

    Ok(OnlineMetadata { genre, year })
}

fn merge_online_metadata(mut base: OnlineMetadata, incoming: OnlineMetadata) -> OnlineMetadata {
    base.genre = metadata::merge_genres(base.genre.as_deref(), incoming.genre.as_deref());
    if base.year.is_none() {
        base.year = incoming.year;
    }
    base
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

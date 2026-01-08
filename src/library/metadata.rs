use std::collections::HashSet;
use std::path::Path;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::picture::PictureType;
use lofty::tag::ItemKey;
use tracing::warn;

use crate::library::EmbeddedCover;

#[derive(Debug, Default, Clone)]
pub struct TrackMetadata {
    pub duration_secs: Option<u32>,
    pub duration_millis: Option<u64>,
    pub bitrate_kbps: Option<u32>,
    pub codec: Option<String>,
    pub genre: Option<String>,
    pub embedded_cover: Option<EmbeddedCover>,
}

pub fn track_metadata(path: &Path) -> TrackMetadata {
    let tagged_file = match lofty::read_from_path(path) {
        Ok(tagged_file) => tagged_file,
        Err(error) => {
            warn!(
                error = %error,
                path = %path.display(),
                "Failed to decode audio metadata"
            );
            return TrackMetadata::default();
        }
    };

    let properties = tagged_file.properties();
    let duration = properties.duration();
    let duration_secs = u32::try_from(duration.as_secs()).ok();
    let duration_millis = u64::try_from(duration.as_millis()).ok();
    let bitrate_kbps = properties
        .audio_bitrate()
        .or_else(|| properties.overall_bitrate())
        .filter(|bitrate| *bitrate > 0);
    let codec = Some(format!("{:?}", tagged_file.file_type()));

    let genre = extract_genre(&tagged_file);

    let embedded_cover = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())
        .and_then(|tag| {
            tag.get_picture_type(PictureType::CoverFront)
                .or_else(|| tag.pictures().first())
        })
        .and_then(|picture| {
            if picture.data().is_empty() {
                return None;
            }
            Some(EmbeddedCover {
                mime_type: picture.mime_type().map(|mime| mime.as_str().to_string()),
                data: picture.data().to_vec(),
            })
        });

    TrackMetadata {
        duration_secs,
        duration_millis,
        bitrate_kbps,
        codec,
        genre,
        embedded_cover,
    }
}

fn extract_genre(tagged_file: &impl TaggedFileExt) -> Option<String> {
    let mut genres = Vec::new();
    let mut seen = HashSet::new();
    let mut tags = Vec::new();

    if let Some(primary_tag) = tagged_file.primary_tag() {
        tags.push(primary_tag);
    }

    tags.extend(tagged_file.tags());

    for tag in tags {
        let values = tag
            .get_strings(&ItemKey::Genre)
            .chain(tag.get_locators(&ItemKey::Genre));

        for value in values {
            for genre in split_genre_field(value) {
                let normalized = genre.to_lowercase();
                if seen.insert(normalized) {
                    genres.push(genre.to_string());
                }
            }
        }
    }

    if genres.is_empty() {
        None
    } else {
        Some(genres.join("; "))
    }
}

fn split_genre_field(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(|ch| matches!(ch, ';' | '/' | '\\' | ',' | '|'))
        .map(|genre| genre.trim())
        .filter(|genre| !genre.is_empty())
}

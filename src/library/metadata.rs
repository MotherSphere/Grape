use std::path::Path;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::picture::PictureType;
use lofty::tag::{ItemKey, TagExt};
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

    let genre = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())
        .and_then(|tag| tag.get_string(&ItemKey::Genre))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

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

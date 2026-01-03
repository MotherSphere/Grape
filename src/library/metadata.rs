use std::path::Path;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{ItemKey, TagExt};
use tracing::warn;

#[derive(Debug, Default, Clone)]
pub struct TrackMetadata {
    pub duration_secs: Option<u32>,
    pub genre: Option<String>,
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

    let duration = tagged_file.properties().duration().as_secs();
    let duration_secs = u32::try_from(duration).ok();

    let genre = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())
        .and_then(|tag| tag.get_string(&ItemKey::Genre))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    TrackMetadata {
        duration_secs,
        genre,
    }
}

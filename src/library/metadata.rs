use std::path::Path;

use lofty::file::AudioFile;
use tracing::warn;

pub fn track_duration_secs(path: &Path) -> Option<u32> {
    let tagged_file = match lofty::read_from_path(path) {
        Ok(tagged_file) => tagged_file,
        Err(error) => {
            warn!(
                error = %error,
                path = %path.display(),
                "Failed to decode audio metadata"
            );
            return None;
        }
    };
    let duration = tagged_file.properties().duration().as_secs();
    u32::try_from(duration).ok()
}

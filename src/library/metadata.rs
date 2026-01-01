use std::path::Path;

pub fn track_duration_secs(path: &Path) -> Option<u32> {
    let tagged_file = lofty::read_from_path(path).ok()?;
    let duration = tagged_file.properties().duration().as_secs();
    u32::try_from(duration).ok()
}

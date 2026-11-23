//! Measuring and deleting what `mash uninstall` removes, without following
//! links and never at or above the user's home directory.
use std::path::Path;

pub(super) fn present(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

/// Bytes under `path`, without following links.
pub(super) fn size(path: &Path) -> u64 {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if !metadata.is_dir() {
        return metadata.len();
    }
    std::fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| size(&entry.path()))
        .sum()
}

/// Delete one file or directory tree. A path that is, or holds, the user's
/// home directory is refused whatever a flag named.
pub(super) fn remove(path: &Path, user: &Path) -> Result<(), String> {
    if !path.is_absolute() || user.starts_with(path) {
        return Err(format!(
            "refusing to delete {}: it is not a location Mash owns",
            path.display()
        ));
    }
    let result = match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(path),
        Ok(_) => std::fs::remove_file(path),
        Err(error) => Err(error),
    };
    match result {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("cannot delete {}: {error}", path.display())),
    }
}

//! The admin key this machine holds for its shared server, if any.
//!
//! It is kept in `admin.env` in Mash's own directory, readable only by the
//! user, like the node token in `node.env`, and never in the configuration
//! file, a URL or a log. Only the dashboard's management routes read it, and
//! it is sent only to the shared server this machine already publishes to.
use crate::AppError;

const NAME: &str = "MASH_ADMIN_KEY";

fn path() -> Result<std::path::PathBuf, AppError> {
    crate::platform::mash_home()
        .map(|home| home.join("admin.env"))
        .ok_or_else(|| AppError::configuration("cannot find the home directory"))
}

/// The key saved on this machine.
pub fn load() -> Option<String> {
    let text = std::fs::read_to_string(path().ok()?).ok()?;
    text.lines()
        .find_map(|line| line.strip_prefix(NAME)?.strip_prefix('='))
        .filter(|key| super::server::admin::well_formed(key))
        .map(str::to_owned)
}

/// Save a key, replacing the one saved before.
pub fn save(key: &str) -> Result<(), AppError> {
    if !super::server::admin::well_formed(key) {
        return Err(AppError::usage(
            "that is not a Mash admin key; copy the whole `mash-admin.…` value",
        ));
    }
    crate::config::save_token(&path()?, NAME, key)
}

/// Forget the saved key. Nothing on the server changes.
pub fn forget() -> Result<(), AppError> {
    match std::fs::remove_file(path()?) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::io(error)),
    }
}

//! Finding an update: reading the channel's `latest.json`, downloading a
//! newer version and keeping it, verified, until it is installed. A check
//! from the menu that finds one brings the main window forward with the
//! banner in it, and otherwise says what it found; a check on the timer says
//! nothing. No check installs anything.

use super::channel::{self, Channel};
use super::{Phase, banner, lock, refresh, release, surface, tell};
use semver::Version;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::{Update, UpdaterExt};

/// Names another `latest.json` in a test build, whose configuration allows a
/// plain-HTTP address; a released build ignores it.
const TEST_ENDPOINT: &str = "MASH_DESKTOP_UPDATE_ENDPOINT";

/// Check for Updates… in a build that does not update itself. A newer
/// stable release is shown in the banner, which offers the downloads page or
/// the apt command. With none, or no window to show it in, a copy that apt
/// installed (the .deb puts the app in /usr/bin) is told how apt brings a
/// new version, and any other copy gets the downloads page.
pub(super) fn check_by_hand(app: &AppHandle) {
    let current = app.package_info().version.clone();
    let newer = release::newer(&current);
    lock(app).offer.newer = newer.as_ref().map(ToString::to_string);
    if newer.is_some() && surface(app) {
        return;
    }
    if !super::from_apt() {
        let _ = app.opener().open_url(channel::DOWNLOADS, None::<&str>);
        return;
    }
    const APT: &str = banner::APT;
    tell(
        app,
        &match newer {
            Some(newer) => format!(
                "Mash {newer} is available; this is {current}. This copy was installed with apt, \
                 which updates the app and the mash command together:\n\n{APT}"
            ),
            None => format!(
                "Mash {current} was installed with apt, which keeps the app and the mash \
                 command up to date together:\n\n{APT}"
            ),
        },
    );
}

/// The channel's address, or the test address in a test build.
fn endpoint(app: &AppHandle, channel: Channel) -> String {
    let test_build = super::updater_config(app, "dangerousInsecureTransportProtocol")
        .is_some_and(|allowed| allowed == true);
    std::env::var(TEST_ENDPOINT)
        .ok()
        .filter(|_| test_build)
        .unwrap_or_else(|| channel.endpoint())
}

/// Read the channel and download a newer version. `asked` is a check from the
/// menu, which shows the banner for a version it finds, or already has, and
/// otherwise says what it found; a check on the timer says nothing.
pub(super) fn check(app: &AppHandle, asked: bool) {
    let (channel, skipped) = {
        let mut inner = lock(app);
        match inner.phase {
            Phase::Idle => {}
            Phase::Ready { .. } | Phase::Downloading(_) if asked => {
                drop(inner);
                return show_offer(app);
            }
            _ => return,
        }
        inner.phase = Phase::Checking;
        refresh(app, &inner);
        let skipped = inner.settings.skipped_version.as_deref();
        (
            inner.settings.channel,
            skipped.and_then(|version| Version::parse(version).ok()),
        )
    };
    let found = find(app, channel, skipped);
    let mut inner = lock(app);
    // Update and Restart, chosen in the banner while the download arrived.
    let wanted = std::mem::take(&mut inner.offer.wanted);
    inner.offer.progress = None;
    match found {
        Ok(Some((update, file))) => {
            inner.phase = Phase::Ready {
                update: Box::new(update),
                file,
            };
            inner.offer.failed = None;
            refresh(app, &inner);
            drop(inner);
            if wanted {
                super::install::apply(app);
            } else if asked {
                show_offer(app);
            }
        }
        Ok(None) => {
            inner.phase = Phase::Idle;
            refresh(app, &inner);
            drop(inner);
            if asked {
                tell(
                    app,
                    &format!(
                        "Mash {} is the newest version on the {} channel.",
                        app.package_info().version,
                        channel.label()
                    ),
                );
            }
        }
        Err(error) => {
            let downloading = match &inner.phase {
                Phase::Downloading(version) => Some(version.clone()),
                _ => None,
            };
            if let Some(version) = downloading.filter(|_| wanted) {
                inner.offer.failed = Some((version, error.clone()));
            }
            inner.phase = Phase::Idle;
            refresh(app, &inner);
            drop(inner);
            eprintln!("{error}");
            if asked {
                tell(app, &error);
            }
        }
    }
}

/// The channel's newer version, downloaded and verified, if it has one.
fn find(
    app: &AppHandle,
    channel: Channel,
    skipped: Option<Version>,
) -> Result<Option<(Update, PathBuf)>, String> {
    let address = endpoint(app, channel)
        .parse()
        .map_err(|error| format!("Mash couldn’t check for updates: {error}"))?;
    let updater = app
        .updater_builder()
        .endpoints(vec![address])
        .map_err(couldnt_check)?
        .version_comparator(move |current, release| {
            channel::offers(channel, &current, &release.version, skipped.as_ref())
        })
        .timeout(Duration::from_secs(30 * 60))
        .build()
        .map_err(couldnt_check)?;
    let update = match tauri::async_runtime::block_on(updater.check()) {
        Ok(Some(update)) => update,
        Ok(None) => return Ok(None),
        // A channel that has no build for this platform has no update for it.
        Err(
            tauri_plugin_updater::Error::TargetNotFound(_)
            | tauri_plugin_updater::Error::TargetsNotFound(..),
        ) => return Ok(None),
        Err(error) => return Err(couldnt_check(error)),
    };
    let folder = app
        .path()
        .app_cache_dir()
        .map_err(|error| format!("Mash couldn’t keep the update: {error}"))?
        .join("updates");
    let file = folder.join(format!("mash-{}", update.version));
    // A download from an earlier run is used again only if it still verifies.
    if std::fs::read(&file).is_ok_and(|bytes| update.verify(&bytes).is_ok()) {
        return Ok(Some((update, file)));
    }
    {
        let mut inner = lock(app);
        inner.phase = Phase::Downloading(update.version.clone());
        refresh(app, &inner);
    }
    // The updater checks the download against the app's public key and
    // hands nothing back when it does not match.
    let mut arrived = 0u64;
    let progress = |chunk: usize, size: Option<u64>| {
        arrived += chunk as u64;
        lock(app).offer.progress = Some((arrived, size));
    };
    let bytes =
        tauri::async_runtime::block_on(update.download(progress, || {})).map_err(|error| {
            format!(
                "Mash did not accept the update it downloaded ({}), and is unchanged: {error}",
                update.version
            )
        })?;
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder)
        .and_then(|()| std::fs::write(&file, bytes))
        .map_err(|error| format!("Mash couldn’t keep the update: {error}"))?;
    Ok(Some((update, file)))
}

fn couldnt_check(error: tauri_plugin_updater::Error) -> String {
    format!("Mash couldn’t check for updates: {error}")
}

/// A check from the menu has an update to offer: the banner offers it. Only
/// when the main window cannot be shown does a notice point at the menu
/// line, which is the same action.
fn show_offer(app: &AppHandle) {
    if surface(app) {
        return;
    }
    let version = match &lock(app).phase {
        Phase::Ready { update, .. } => update.version.clone(),
        Phase::Downloading(version) => version.clone(),
        _ => return,
    };
    tell(
        app,
        &format!("Mash {version} is available. Choose Update and Restart in the menu."),
    );
}

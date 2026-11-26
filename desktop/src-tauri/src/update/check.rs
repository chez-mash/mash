//! Finding an update: reading the channel's `latest.json`, downloading a
//! newer version and keeping it, verified, until it is installed. A check
//! from the menu says what it found; a check on the timer says nothing.

use super::channel::{self, Channel};
use super::{Phase, lock, refresh, tell};
use semver::Version;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::{Update, UpdaterExt};

/// Names another `latest.json` in a test build, whose configuration allows a
/// plain-HTTP address; a released build ignores it.
const TEST_ENDPOINT: &str = "MASH_DESKTOP_UPDATE_ENDPOINT";

/// Check for Updates… in a build that does not update itself. A copy that
/// apt installed (the .deb puts the app in /usr/bin) is told whether the
/// stable channel has a newer release and how apt brings it; any other copy
/// gets the downloads page.
pub(super) fn check_by_hand(app: &AppHandle) {
    let from_apt = cfg!(target_os = "linux")
        && std::env::current_exe().is_ok_and(|program| program.starts_with("/usr/bin"));
    if !from_apt {
        let _ = app.opener().open_url(channel::DOWNLOADS, None::<&str>);
        return;
    }
    const APT: &str = "sudo apt update && sudo apt install --only-upgrade mash-desktop mash-cli";
    let current = app.package_info().version.clone();
    let newer = Channel::Stable
        .endpoint()
        .parse()
        .ok()
        .and_then(|address| app.updater_builder().endpoints(vec![address]).ok())
        .and_then(|builder| builder.build().ok())
        .and_then(|updater| {
            tauri::async_runtime::block_on(updater.check())
                .ok()
                .flatten()
        });
    tell(
        app,
        &match newer {
            Some(update) => format!(
                "Mash {} is available; this is {current}. This copy was installed with apt, \
                 which updates the app and the mash command together:\n\n{APT}",
                update.version
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
/// menu, which says what it found; a check on the timer says nothing.
pub(super) fn check(app: &AppHandle, asked: bool) {
    let (channel, skipped) = {
        let mut inner = lock(app);
        match inner.phase {
            Phase::Idle => {}
            Phase::Ready { .. } if asked => {
                drop(inner);
                return offer_restart(app);
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
    match found {
        Ok(Some((update, file))) => {
            inner.phase = Phase::Ready {
                update: Box::new(update),
                file,
            };
            inner.idle_ticks = 0;
            refresh(app, &inner);
            drop(inner);
            if asked {
                offer_restart(app);
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
    let bytes =
        tauri::async_runtime::block_on(update.download(|_, _| {}, || {})).map_err(|error| {
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

fn offer_restart(app: &AppHandle) {
    let version = match &lock(app).phase {
        Phase::Ready { update, .. } => update.version.clone(),
        _ => return,
    };
    let handle = app.clone();
    app.dialog()
        .message(format!("Mash {version} is ready to install."))
        .title("Mash")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Restart to Update".into(),
            "Later".into(),
        ))
        .show(move |restart| {
            if restart {
                std::thread::spawn(move || super::install::apply(&handle, false));
            }
        });
}

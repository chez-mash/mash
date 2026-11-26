//! Installing a ready update and what follows it: the engine stops, the
//! installed app is copied aside, the updater replaces it and the new
//! version starts. The first start of the new version upgrades the index
//! before its engine opens it, and once that engine is up the copy goes,
//! Mash's services restart and anything still on the earlier version is
//! named. If that engine does not start, the menus offer the way back.

use super::settings::Pending;
use super::{
    Phase, after, installed, keeps_separate, kept_folder, lock, previous, refresh, save, tell,
};
use crate::{engine, lockstep};
use std::time::Duration;
use tauri::AppHandle;

/// Install a ready update while nobody is looking: on, the main window and
/// the search panel both hidden at two looks in a row, so a search in either
/// is never cut off.
pub(super) fn install_if_idle(app: &AppHandle) {
    let mut inner = lock(app);
    let ready = matches!(inner.phase, Phase::Ready { .. });
    if !ready || !inner.settings.installs_when_idle() || !crate::nothing_showing(app) {
        inner.idle_ticks = 0;
        return;
    }
    inner.idle_ticks += 1;
    if inner.idle_ticks >= 2 {
        drop(inner);
        apply(app, true);
    }
}

/// Install the ready update and start the new version. The engine is stopped
/// first and the installed app copied aside; if the install fails the engine
/// starts again and the app carries on as it was.
pub(super) fn apply(app: &AppHandle, quiet: bool) {
    let (update, file) = {
        let mut inner = lock(app);
        let Phase::Ready { update, .. } = &inner.phase else {
            return;
        };
        let version = update.version.clone();
        let Phase::Ready { update, file } =
            std::mem::replace(&mut inner.phase, Phase::Installing(version))
        else {
            return;
        };
        refresh(app, &inner);
        (update, file)
    };
    let failed = |why: String| {
        let _ = std::fs::remove_file(&file);
        let mut inner = lock(app);
        inner.phase = Phase::Idle;
        inner.settings.pending = None;
        save(app, &inner.settings);
        refresh(app, &inner);
        drop(inner);
        eprintln!("Mash couldn’t install the update: {why}");
        if !quiet {
            tell(
                app,
                &format!("Mash couldn’t install the update, and is unchanged: {why}"),
            );
        }
    };
    // Verified before anything is stopped: a download changed on disk is
    // refused here, with the app untouched.
    let bytes = match std::fs::read(&file) {
        Ok(bytes) => bytes,
        Err(error) => return failed(format!("the download is gone ({error})")),
    };
    if let Err(error) = update.verify(&bytes) {
        return failed(format!(
            "the download did not pass its signature check ({error})"
        ));
    }
    crate::stop_engine(app);
    let kept = installed()
        .zip(kept_folder(app))
        .and_then(|(installed, folder)| previous::keep(&installed, &folder).ok());
    {
        let mut inner = lock(app);
        inner.settings.pending = Some(Pending {
            from: app.package_info().version.to_string(),
            to: update.version.clone(),
            previous: kept,
            quiet,
        });
        save(app, &inner.settings);
    }
    // On Windows this hands over to the installer and ends the app itself.
    if let Err(error) = update.install(&bytes) {
        if let Some(folder) = kept_folder(app) {
            previous::discard(&folder);
        }
        failed(error.to_string());
        crate::restart_engine(app);
        return;
    }
    let _ = std::fs::remove_file(&file);
    app.restart();
}

/// Before the engine starts: the first start after an update brings the
/// index up to this version.
pub fn before_engine(app: &AppHandle, log: &engine::Log) {
    if lock(app).arrived.is_some()
        && let Ok(program) = engine::locate()
    {
        after::upgrade_index(&program, log);
    }
}

/// The engine is up. After an update that settles it: the kept copy goes,
/// the services that run the app's mash restart, and whatever still runs the
/// earlier version is named. On an ordinary start, look for another mash.
pub fn engine_started(app: &AppHandle) {
    let arrived = {
        let mut inner = lock(app);
        let arrived = inner.arrived.take();
        if arrived.is_some() {
            inner.settings.pending = None;
            save(app, &inner.settings);
        }
        arrived
    };
    let app = app.clone();
    std::thread::spawn(move || {
        let Some(arrived) = arrived else {
            return lockstep::at_launch(&app);
        };
        if let Some(folder) = kept_folder(&app) {
            previous::discard(&folder);
        }
        after::restart_services(&lockstep::programs());
        // Give restarted services a moment to register themselves.
        std::thread::sleep(Duration::from_secs(5));
        let separate = keeps_separate(&app);
        let Some(notice) = lockstep::after_update(&arrived.to, separate) else {
            return;
        };
        if arrived.quiet {
            let mut inner = lock(&app);
            inner.notice = Some(notice);
            refresh(&app, &inner);
        } else {
            tell(&app, &notice);
        }
    });
}

/// The engine did not start. Right after an update, with the previous app
/// still kept, the menus offer the way back.
pub fn engine_failed(app: &AppHandle) {
    let mut inner = lock(app);
    let kept = inner
        .arrived
        .as_ref()
        .is_some_and(|pending| pending.previous.is_some());
    if kept {
        inner.items.offer_going_back(app);
    }
}

/// Put the previous app back and start it; the version gone back from is
/// not installed again.
pub(super) fn go_back(app: &AppHandle) {
    let Some((pending, installed)) = lock(app).arrived.clone().zip(installed()) else {
        return;
    };
    let Some(kept) = pending.previous.clone() else {
        return;
    };
    let app = app.clone();
    std::thread::spawn(move || {
        crate::stop_engine(&app);
        if let Err(error) = previous::restore(&kept, &installed) {
            tell(
                &app,
                &format!("Mash couldn’t go back to version {}: {error}", pending.from),
            );
            return crate::restart_engine(&app);
        }
        {
            let mut inner = lock(&app);
            inner.settings.skipped_version = Some(pending.to.clone());
            inner.settings.pending = None;
            save(&app, &inner.settings);
        }
        app.restart();
    });
}

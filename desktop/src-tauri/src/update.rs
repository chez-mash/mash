//! The app updates itself: it reads its channel's `latest.json`, downloads a
//! newer version in the background, checks it against the public key built
//! into the app, and installs it when the person chooses Restart to Update
//! (or, with the idle option, while no window is showing).
//!
//! Nothing interrupts: a ready update is one line in the menus. Updates are
//! off, and Check for Updates… opens the downloads page instead, when the
//! build has no public key or is not a kind the updater can replace (a
//! development build, a .deb). The download is verified when it arrives and
//! again from disk just before it is installed, so a file changed in between
//! is refused. The app that was replaced is kept until the new version's
//! engine has started (previous.rs); what the new version then needs is in
//! after.rs.

pub mod after;
mod channel;
mod check;
mod install;
mod menu;
mod previous;
mod settings;

use channel::Channel;
pub use install::{before_engine, engine_failed, engine_started};
pub use menu::Holder;
use settings::{Pending, Settings};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_updater::Update;

/// How often the channel is read, after the check at launch.
const EVERY: Duration = Duration::from_secs(6 * 60 * 60);
/// How often the app looks whether it is idle with an update ready.
const TICK: Duration = Duration::from_secs(30);

/// Where an update stands.
enum Phase {
    /// This build does not update itself.
    Off,
    Idle,
    Checking,
    Downloading(String),
    /// Downloaded, verified and waiting in `file`.
    Ready {
        update: Box<Update>,
        file: PathBuf,
    },
    Installing(String),
}

struct Inner {
    settings: Settings,
    phase: Phase,
    /// The update this run arrived by, until its engine has started.
    arrived: Option<Pending>,
    /// What does not yet run the new version, shown when the line is chosen.
    notice: Option<String>,
    /// Ticks in a row with no window showing.
    idle_ticks: u8,
    items: menu::Items,
}

pub struct Updates(Mutex<Inner>);

fn lock(app: &AppHandle) -> MutexGuard<'_, Inner> {
    app.state::<Updates>()
        .inner()
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

fn settings_file(app: &AppHandle) -> Option<PathBuf> {
    Some(app.path().app_config_dir().ok()?.join("updates.json"))
}

fn kept_folder(app: &AppHandle) -> Option<PathBuf> {
    Some(app.path().app_data_dir().ok()?.join("previous"))
}

fn save(app: &AppHandle, settings: &Settings) {
    if let Some(file) = settings_file(app)
        && let Err(error) = settings.save(&file)
    {
        eprintln!("Mash couldn’t save its update settings: {error}");
    }
}

/// The app this run would replace, if it is one the updater can.
fn installed() -> Option<PathBuf> {
    previous::installed(
        &std::env::current_exe().ok()?,
        std::env::var_os("APPIMAGE").map(PathBuf::from),
    )
}

fn updater_config(app: &AppHandle, key: &str) -> Option<serde_json::Value> {
    app.config().plugins.0.get("updater")?.get(key).cloned()
}

/// Whether this build updates itself: it carries a public key and is
/// installed as something the updater replaces, which is the app bundle on
/// macOS and an AppImage on Linux. A .deb is updated by apt. Windows is off
/// until the installer's handover (stop everything that runs mash.exe, swap,
/// start) has been verified on a real machine.
fn enabled(app: &AppHandle) -> bool {
    let keyed = updater_config(app, "pubkey")
        .is_some_and(|key| key.as_str().is_some_and(|key| !key.trim().is_empty()));
    keyed && installed().is_some() && !cfg!(debug_assertions)
}

/// Load the settings and settle what the last run left: called before the
/// window opens, so a quiet update can start without one.
pub fn prepare(app: &AppHandle) {
    let mut settings = settings_file(app)
        .map(|file| Settings::load(&file))
        .unwrap_or_default();
    let current = app.package_info().version.to_string();
    let arrived = settings
        .pending
        .take()
        .filter(|pending| pending.to == current);
    if arrived.is_none() {
        // No update, or one that did not take, or one gone back from: the
        // kept copy has no further use.
        if let Some(folder) = kept_folder(app) {
            previous::discard(&folder);
        }
        save(app, &settings);
    }
    let phase = if enabled(app) {
        Phase::Idle
    } else {
        Phase::Off
    };
    app.manage(Updates(Mutex::new(Inner {
        settings,
        phase,
        arrived,
        notice: None,
        idle_ticks: 0,
        items: menu::Items::default(),
    })));
}

/// Whether this start follows an update installed while the app was idle,
/// which leaves the window closed as it was.
pub fn starts_quietly(app: &AppHandle) -> bool {
    lock(app)
        .arrived
        .as_ref()
        .is_some_and(|pending| pending.quiet)
}

pub fn keeps_separate(app: &AppHandle) -> bool {
    lock(app).settings.keep_separate
}

pub fn keep_separate(app: &AppHandle) {
    let mut inner = lock(app);
    inner.settings.keep_separate = true;
    save(app, &inner.settings);
}

/// Add the update items to one of the app's menus.
pub fn add_menu(app: &AppHandle, holder: Holder, position: Option<usize>) -> tauri::Result<()> {
    let mut inner = lock(app);
    let channels = !matches!(inner.phase, Phase::Off);
    inner.items.add_to(app, holder, position, channels)?;
    refresh(app, &inner);
    Ok(())
}

/// Bring the menus up to date with the state.
fn refresh(app: &AppHandle, inner: &Inner) {
    let version = &app.package_info().version;
    let (status, actionable) = match &inner.phase {
        Phase::Ready { update, .. } => (
            format!("Update Ready ({}) — Restart to Update", update.version),
            true,
        ),
        Phase::Installing(version) => (format!("Updating to {version}…"), false),
        Phase::Downloading(version) => (format!("Downloading Mash {version}…"), false),
        _ if inner.notice.is_some() => (
            format!("Updated to {version} — See What Needs Restarting…"),
            true,
        ),
        Phase::Checking => ("Checking for Updates…".to_owned(), false),
        Phase::Idle | Phase::Off => (format!("Mash {version}"), false),
    };
    inner.items.show(&menu::View {
        status,
        actionable,
        channel: inner.settings.channel,
        installs_when_idle: inner.settings.installs_when_idle(),
    });
}

/// Start checking: now, then every six hours, and look every half minute
/// whether a ready update may install because the app is idle.
pub fn start(app: &AppHandle) {
    if matches!(lock(app).phase, Phase::Off) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        loop {
            check::check(&app, false);
            let mut waited = Duration::ZERO;
            while waited < EVERY {
                std::thread::sleep(TICK);
                waited += TICK;
                install::install_if_idle(&app);
            }
        }
    });
}

/// Act on a menu choice; false when the choice is not about updates.
pub fn choose(app: &AppHandle, id: &str) -> bool {
    match id {
        menu::CHECK => {
            let off = matches!(lock(app).phase, Phase::Off);
            let app = app.clone();
            std::thread::spawn(move || {
                if off {
                    check::check_by_hand(&app)
                } else {
                    check::check(&app, true)
                }
            });
        }
        menu::STATUS => status_chosen(app),
        menu::STABLE => switch(app, Channel::Stable),
        menu::MAIN => switch(app, Channel::Main),
        menu::IDLE => {
            let mut inner = lock(app);
            inner.settings.install_when_idle = Some(!inner.settings.installs_when_idle());
            save(app, &inner.settings);
            refresh(app, &inner);
        }
        menu::BACK => install::go_back(app),
        _ => return false,
    }
    true
}

/// The status line was chosen: restart into the ready update, or show what
/// the last update left behind.
fn status_chosen(app: &AppHandle) {
    let mut inner = lock(app);
    if matches!(inner.phase, Phase::Ready { .. }) {
        drop(inner);
        let app = app.clone();
        std::thread::spawn(move || install::apply(&app, false));
    } else if let Some(notice) = inner.notice.take() {
        refresh(app, &inner);
        drop(inner);
        tell(app, &notice);
    }
}

fn switch(app: &AppHandle, channel: Channel) {
    let mut inner = lock(app);
    if inner.settings.channel != channel {
        inner.settings.channel = channel;
        save(app, &inner.settings);
        // What was downloaded belongs to the other channel.
        if let Phase::Ready { file, .. } = std::mem::replace(&mut inner.phase, Phase::Idle) {
            let _ = std::fs::remove_file(file);
        }
        let app = app.clone();
        std::thread::spawn(move || check::check(&app, false));
    }
    refresh(app, &inner);
}

fn tell(app: &AppHandle, message: &str) {
    app.dialog().message(message).title("Mash").show(|_| {});
}

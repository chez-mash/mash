//! The update banner in the main window. The dashboard draws it; the app
//! says what there is to offer and does what its two buttons ask, through
//! three commands that only the main window may call (capabilities/
//! updates.json). Nothing installs by itself: an update is installed only
//! when the person chooses Update and Restart, here or on the menu line,
//! which is the same request (`install::request`).
//!
//! A build that updates itself offers the version it has downloaded, or is
//! downloading. A
//! build that does not (no public key, a .deb, Windows, a development build)
//! offers the newer stable release GitHub names (release.rs) with the
//! downloads page, or with the apt command for a copy apt installed.
//! Later hides the banner for that version until the app starts again.

use super::channel::DOWNLOADS;
use super::install;
use super::{Phase, lock};
use serde::Serialize;
use tauri::AppHandle;

/// How apt brings a newer version to a copy it installed.
pub(super) const APT: &str =
    "sudo apt update && sudo apt install --only-upgrade mash-desktop mash-cli";

/// What the banner has been told and asked, beside the update's own phase.
#[derive(Default)]
pub(super) struct Offer {
    /// The newer stable release, in a build that does not update itself.
    pub newer: Option<String>,
    /// Update and Restart was chosen while the download was still arriving.
    pub wanted: bool,
    /// How much of the download has arrived, and its size when known.
    pub progress: Option<(u64, Option<u64>)>,
    /// The version that did not install, and why.
    pub failed: Option<(String, String)>,
    /// The version Later was chosen for.
    pub later: Option<String>,
}

/// How the person gets the version on offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Way {
    /// The app installs it and restarts.
    Install,
    /// The downloads page.
    Download,
    /// The apt command.
    Apt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Stage {
    Available,
    Downloading,
    Installing,
    Failed,
}

/// What the dashboard draws.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Banner {
    /// The running version and the one on offer.
    current: String,
    version: String,
    way: Way,
    stage: Stage,
    /// The part of the download that has arrived, from 0 to 1, when known.
    progress: Option<f64>,
    /// Why the update did not install.
    reason: Option<String>,
    /// The apt command, for a copy apt installed.
    command: Option<&'static str>,
    /// Where a person downloads Mash by hand.
    downloads: &'static str,
}

/// Where the update stands, without the update itself.
#[derive(Debug, Clone, Copy)]
pub(super) enum Standing<'a> {
    /// This build does not update itself.
    Off,
    Nothing,
    Downloading(&'a str),
    Ready(&'a str),
    Installing(&'a str),
}

impl<'a> Standing<'a> {
    pub(super) fn of(phase: &'a Phase) -> Self {
        match phase {
            Phase::Off => Self::Off,
            Phase::Idle | Phase::Checking => Self::Nothing,
            Phase::Downloading(version) => Self::Downloading(version),
            Phase::Ready { update, .. } => Self::Ready(&update.version),
            Phase::Installing(version) => Self::Installing(version),
        }
    }
}

/// The banner to draw, if there is anything to offer.
pub(super) fn banner(
    standing: Standing<'_>,
    offer: &Offer,
    current: &str,
    from_apt: bool,
) -> Option<Banner> {
    let (version, way, stage) = match standing {
        // An install under way is shown whoever asked for it.
        Standing::Installing(version) => (version, Way::Install, Stage::Installing),
        Standing::Downloading(version) if offer.wanted => {
            (version, Way::Install, Stage::Downloading)
        }
        Standing::Downloading(version) | Standing::Ready(version) => {
            (version, Way::Install, Stage::Available)
        }
        Standing::Off | Standing::Nothing => match (&offer.failed, &offer.newer) {
            (Some((version, _)), _) => (version.as_str(), Way::Install, Stage::Failed),
            (None, Some(version)) if matches!(standing, Standing::Off) => {
                let way = if from_apt { Way::Apt } else { Way::Download };
                (version.as_str(), way, Stage::Available)
            }
            _ => return None,
        },
    };
    let waiting = matches!(stage, Stage::Available | Stage::Failed);
    if waiting && offer.later.as_deref() == Some(version) {
        return None;
    }
    let progress = offer
        .progress
        .filter(|_| stage == Stage::Downloading)
        .and_then(|(arrived, size)| Some(arrived as f64 / size.filter(|size| *size > 0)? as f64))
        .map(|part| part.clamp(0.0, 1.0));
    Some(Banner {
        current: current.to_owned(),
        version: version.to_owned(),
        way,
        stage,
        progress,
        reason: offer
            .failed
            .as_ref()
            .filter(|_| stage == Stage::Failed)
            .map(|(_, reason)| reason.clone()),
        command: (way == Way::Apt).then_some(APT),
        downloads: DOWNLOADS,
    })
}

/// What the main window's banner shows now.
#[tauri::command]
pub fn update_banner(app: AppHandle) -> Option<Banner> {
    let inner = lock(&app);
    banner(
        Standing::of(&inner.phase),
        &inner.offer,
        &app.package_info().version.to_string(),
        super::from_apt(),
    )
}

/// Update and Restart.
#[tauri::command]
pub fn update_install(app: AppHandle) {
    install::request(&app);
}

/// Later: hide the banner for the version it shows until the app starts
/// again; a newer version brings it back sooner.
#[tauri::command]
pub fn update_later(app: AppHandle) {
    let mut inner = lock(&app);
    let shown = banner(
        Standing::of(&inner.phase),
        &inner.offer,
        "",
        super::from_apt(),
    );
    if let Some(shown) = shown {
        inner.offer.later = Some(shown.version);
        inner.offer.failed = None;
        inner.offer.wanted = false;
        super::refresh(&app, &inner);
    }
}

#[cfg(test)]
mod tests {
    use super::{APT, Offer, Stage, Standing, Way, banner};

    fn offer() -> Offer {
        Offer::default()
    }

    fn shown(standing: Standing<'_>, offer: &Offer) -> Option<(String, Way, Stage)> {
        banner(standing, offer, "0.2.4", false).map(|b| (b.version, b.way, b.stage))
    }

    #[test]
    fn nothing_newer_shows_nothing() {
        assert_eq!(shown(Standing::Nothing, &offer()), None);
        assert_eq!(shown(Standing::Off, &offer()), None);
        // A release found by a build that has since begun updating itself
        // is the updater's to offer.
        let stale = Offer {
            newer: Some("0.2.5".into()),
            ..offer()
        };
        assert_eq!(shown(Standing::Nothing, &stale), None);
    }

    #[test]
    fn an_updating_build_offers_to_install_what_it_downloads() {
        let install = |stage| Some(("0.2.5".to_owned(), Way::Install, stage));
        assert_eq!(
            shown(Standing::Downloading("0.2.5"), &offer()),
            install(Stage::Available)
        );
        assert_eq!(
            shown(Standing::Ready("0.2.5"), &offer()),
            install(Stage::Available)
        );
        let wanted = Offer {
            wanted: true,
            progress: Some((30, Some(120))),
            ..offer()
        };
        let downloading = banner(Standing::Downloading("0.2.5"), &wanted, "0.2.4", false).unwrap();
        assert_eq!(downloading.stage, Stage::Downloading);
        assert_eq!(downloading.progress, Some(0.25));
        assert_eq!(downloading.current, "0.2.4");
        assert_eq!(
            shown(Standing::Installing("0.2.5"), &offer()),
            install(Stage::Installing)
        );
    }

    #[test]
    fn a_download_of_unknown_size_has_no_progress() {
        for progress in [None, Some((30, None)), Some((30, Some(0)))] {
            let wanted = Offer {
                wanted: true,
                progress,
                ..offer()
            };
            let shown = banner(Standing::Downloading("0.2.5"), &wanted, "0.2.4", false).unwrap();
            assert_eq!(shown.progress, None);
        }
    }

    #[test]
    fn a_build_that_does_not_update_itself_points_at_the_download() {
        let newer = Offer {
            newer: Some("0.2.5".into()),
            ..offer()
        };
        let page = banner(Standing::Off, &newer, "0.2.4", false).unwrap();
        assert_eq!((page.way, page.stage), (Way::Download, Stage::Available));
        assert_eq!(page.command, None);
        assert_eq!(
            page.downloads,
            "https://github.com/mashhq/mash/releases/latest"
        );
        let apt = banner(Standing::Off, &newer, "0.2.4", true).unwrap();
        assert_eq!((apt.way, apt.command), (Way::Apt, Some(APT)));
    }

    #[test]
    fn a_failure_is_shown_with_its_reason() {
        let failed = Offer {
            failed: Some(("0.2.5".into(), "the download is gone".into())),
            ..offer()
        };
        let shown = banner(Standing::Nothing, &failed, "0.2.4", false).unwrap();
        assert_eq!(shown.stage, Stage::Failed);
        assert_eq!(shown.reason.as_deref(), Some("the download is gone"));
        // The next download of the same version is offered again.
        let again = banner(Standing::Ready("0.2.5"), &failed, "0.2.4", false).unwrap();
        assert_eq!((again.stage, again.reason), (Stage::Available, None));
    }

    #[test]
    fn later_hides_one_version_and_never_an_install_under_way() {
        let later = Offer {
            newer: Some("0.2.5".into()),
            later: Some("0.2.5".into()),
            failed: Some(("0.2.5".into(), "no room".into())),
            ..offer()
        };
        assert_eq!(shown(Standing::Off, &later), None);
        assert_eq!(shown(Standing::Nothing, &later), None);
        assert_eq!(shown(Standing::Ready("0.2.5"), &later), None);
        assert!(shown(Standing::Ready("0.2.6"), &later).is_some());
        assert!(shown(Standing::Installing("0.2.5"), &later).is_some());
        let next = Offer {
            newer: Some("0.2.6".into()),
            later: Some("0.2.5".into()),
            ..offer()
        };
        assert!(shown(Standing::Off, &next).is_some());
    }
}

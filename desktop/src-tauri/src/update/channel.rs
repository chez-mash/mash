//! The two update channels and which versions each one offers.
//!
//! `stable` follows the latest release; `main` follows a rolling prerelease
//! that every successful build of the main branch replaces. Both publish the
//! same `latest.json`, so the only difference in the app is the address it
//! reads. A build of main is versioned as a prerelease of the release that
//! will follow it (`0.2.5-main.118` after 0.2.4), so by ordinary semantic
//! version order it is newer than the release before it, older than the
//! release after it, and older than the next build of main.

use semver::Version;
use serde::{Deserialize, Serialize};

const RELEASES: &str = "https://github.com/mashhq/mash/releases";
/// Where a person downloads Mash by hand.
pub const DOWNLOADS: &str = "https://github.com/mashhq/mash/releases/latest";
/// The tag of the rolling prerelease that holds the newest build of main.
pub const MAIN_TAG: &str = "main-channel";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    #[default]
    Stable,
    Main,
}

impl Channel {
    /// The `latest.json` this channel reads.
    pub fn endpoint(self) -> String {
        match self {
            Self::Stable => format!("{RELEASES}/latest/download/latest.json"),
            Self::Main => format!("{RELEASES}/download/{MAIN_TAG}/latest.json"),
        }
    }

    /// The channel's name in the menu.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::Main => "Main (latest build)",
        }
    }

    /// Whether an update installs by itself while the app is idle, unless the
    /// person has chosen otherwise. Main changes many times a day, so there it
    /// does; a release waits to be asked.
    pub const fn installs_when_idle(self) -> bool {
        matches!(self, Self::Main)
    }
}

/// Whether `version` is a build of main rather than a release.
pub fn is_main_build(version: &Version) -> bool {
    version
        .pre
        .as_str()
        .split('.')
        .any(|identifier| identifier == "main")
}

/// Whether `candidate`, announced by `channel`, should replace `current`.
///
/// Only a newer version is offered, so nothing ever moves a person backwards:
/// someone who leaves main for stable keeps their build until the release
/// that follows it. The stable channel never offers a build of main, whatever
/// its address serves. `skipped` is a version the person went back from; it
/// is not offered again, though the one after it is.
pub fn offers(
    channel: Channel,
    current: &Version,
    candidate: &Version,
    skipped: Option<&Version>,
) -> bool {
    if channel == Channel::Stable && is_main_build(candidate) {
        return false;
    }
    candidate > current && skipped != Some(candidate)
}

#[cfg(test)]
mod tests {
    use super::{Channel, is_main_build, offers};
    use semver::Version;

    fn version(text: &str) -> Version {
        Version::parse(text).unwrap()
    }

    fn offered(channel: Channel, current: &str, candidate: &str) -> bool {
        offers(channel, &version(current), &version(candidate), None)
    }

    #[test]
    fn each_channel_reads_its_own_release() {
        assert_eq!(
            Channel::Stable.endpoint(),
            "https://github.com/mashhq/mash/releases/latest/download/latest.json"
        );
        assert_eq!(
            Channel::Main.endpoint(),
            "https://github.com/mashhq/mash/releases/download/main-channel/latest.json"
        );
        assert_eq!(Channel::default(), Channel::Stable);
        assert!(Channel::Main.installs_when_idle());
        assert!(!Channel::Stable.installs_when_idle());
    }

    #[test]
    fn a_release_moves_to_main_and_main_to_the_next_release() {
        // Stable to main: the build after 0.2.4 is newer than 0.2.4.
        assert!(offered(Channel::Main, "0.2.4", "0.2.5-main.118"));
        // Main to the next build of main, compared as numbers, not text.
        assert!(offered(Channel::Main, "0.2.5-main.118", "0.2.5-main.119"));
        assert!(offered(Channel::Main, "0.2.5-main.9", "0.2.5-main.10"));
        // Main to the release that follows it, on either channel.
        assert!(offered(Channel::Main, "0.2.5-main.118", "0.2.5"));
        assert!(offered(Channel::Stable, "0.2.5-main.118", "0.2.5"));
        assert!(offered(Channel::Stable, "0.2.5-main.118", "0.3.0"));
        // And on from that release to the builds after it.
        assert!(offered(Channel::Main, "0.2.5", "0.2.6-main.120"));
        // A prerelease release keeps its own order with its builds of main.
        assert!(offered(Channel::Main, "0.3.0-alpha", "0.3.0-alpha.main.7"));
        assert!(offered(Channel::Main, "0.3.0-alpha.main.7", "0.3.0-beta"));
        assert!(offered(Channel::Stable, "0.3.0-alpha.main.7", "0.3.0"));
    }

    #[test]
    fn nothing_moves_backwards_or_sideways() {
        assert!(!offered(Channel::Stable, "0.2.4", "0.2.4"));
        assert!(!offered(Channel::Main, "0.2.5-main.118", "0.2.5-main.118"));
        assert!(!offered(Channel::Main, "0.2.5-main.118", "0.2.5-main.117"));
        // Back on stable, a build of main waits for the release after it.
        assert!(!offered(Channel::Stable, "0.2.5-main.118", "0.2.4"));
        // A release already ahead of the rolling build is not replaced by it.
        assert!(!offered(Channel::Main, "0.2.5", "0.2.5-main.118"));
    }

    #[test]
    fn stable_never_takes_a_build_of_main() {
        assert!(!offered(Channel::Stable, "0.2.4", "0.2.5-main.118"));
        assert!(is_main_build(&version("0.2.5-main.118")));
        assert!(is_main_build(&version("0.3.0-alpha.main.7")));
        assert!(!is_main_build(&version("0.2.5")));
        assert!(!is_main_build(&version("0.3.0-alpha")));
        assert!(!is_main_build(&version("0.3.0-mainline.1")));
    }

    #[test]
    fn a_version_gone_back_from_is_not_offered_again() {
        let skipped = version("0.2.5-main.118");
        let current = version("0.2.5-main.117");
        assert!(!offers(Channel::Main, &current, &skipped, Some(&skipped)));
        assert!(offers(
            Channel::Main,
            &current,
            &version("0.2.5-main.119"),
            Some(&skipped)
        ));
    }
}

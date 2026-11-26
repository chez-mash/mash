//! What a build that does not update itself can still learn: whether the
//! stable channel has a newer release. It reads GitHub's public record of the
//! latest release, which names only the version, so nothing is downloaded and
//! nothing is installed; the person is shown the downloads page, or the apt
//! command, and updates the way they installed.

use super::channel::{self, Channel};
use semver::Version;
use serde::Deserialize;
use std::time::Duration;

/// GitHub's record of the latest release: never a draft or a prerelease, so
/// never a build of main.
const LATEST: &str = "https://api.github.com/repos/mashhq/mash/releases/latest";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

/// The version GitHub's record of a release names; a release is tagged
/// `v0.2.5`.
fn version(record: &str) -> Option<Version> {
    let release: Release = serde_json::from_str(record).ok()?;
    let tag = release.tag_name.trim();
    Version::parse(tag.strip_prefix('v').unwrap_or(tag)).ok()
}

/// The release `record` names, when the stable channel would offer it to
/// `current`: the same order the updater uses.
fn offered(current: &Version, record: &str) -> Option<Version> {
    version(record).filter(|candidate| channel::offers(Channel::Stable, current, candidate, None))
}

/// The stable release newer than `current`, if GitHub names one. A check
/// that fails says nothing and is tried again at the next one.
pub(super) fn newer(current: &Version) -> Option<Version> {
    // The updater's client is built without a TLS provider of its own and
    // installs this one before its first request; this may run before it.
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
    let record = tauri::async_runtime::block_on(async {
        reqwest::Client::builder()
            .user_agent(concat!("mash-desktop/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .build()
            .ok()?
            .get(LATEST)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?
            .text()
            .await
            .ok()
    })?;
    offered(current, &record)
}

#[cfg(test)]
mod tests {
    use super::{offered, version};
    use semver::Version;

    fn record(tag: &str) -> String {
        format!(r#"{{"tag_name":"{tag}","name":"Mash","prerelease":false,"assets":[]}}"#)
    }

    fn current(text: &str) -> Version {
        Version::parse(text).unwrap()
    }

    #[test]
    fn a_release_is_named_by_its_tag() {
        assert_eq!(version(&record("v0.2.5")), Some(current("0.2.5")));
        assert_eq!(version(&record("0.3.0")), Some(current("0.3.0")));
        assert_eq!(version(&record("main-channel")), None);
        assert_eq!(version("not a record"), None);
        assert_eq!(version(r#"{"message":"rate limit exceeded"}"#), None);
    }

    #[test]
    fn only_a_newer_release_is_offered() {
        let newer = Some(current("0.2.5"));
        assert_eq!(offered(&current("0.2.4"), &record("v0.2.5")), newer);
        assert_eq!(offered(&current("0.2.5"), &record("v0.2.5")), None);
        assert_eq!(offered(&current("0.2.6"), &record("v0.2.5")), None);
        // A build of main takes the release that follows it, and nothing
        // moves from a release to a build of main.
        assert_eq!(
            offered(&current("0.2.5-main.118"), &record("v0.2.5")),
            newer
        );
        assert_eq!(offered(&current("0.2.4"), &record("v0.2.5-main.118")), None);
    }
}

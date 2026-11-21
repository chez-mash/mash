use super::super::{Client, Component, State, read};
use serde::Deserialize;
use std::{cmp::Ordering, path::Path};

const REINSTALL_PLUGIN: &str = "Reinstall the Mash plugin through your agent, then restart it.";
const UPDATE_PLUGIN: &str = "Update the Mash plugin through your agent, then restart it.";
const UPDATE_CLI: &str = "Update the Mash CLI when a release matching the installed plugin is available, then restart the agent.";

#[derive(Deserialize)]
pub(in crate::core::installation) struct Stamp {
    pub(in crate::core::installation) contract_fingerprint: String,
    pub(in crate::core::installation) plugin_version: String,
}

pub(in crate::core::installation) fn expected_stamp() -> Stamp {
    // Generated alongside the bundled skill. Invalid build assets fail the checks.
    serde_json::from_str(include_str!(
        "../../../../plugins/mash/skills/mash-history/contract.json"
    ))
    .unwrap_or(Stamp {
        contract_fingerprint: String::new(),
        plugin_version: String::new(),
    })
}

pub(super) struct Version {
    installed: String,
    order: Ordering,
}

/// Mash publishes plugins with three numeric components.
fn components(version: &str) -> Option<[u64; 3]> {
    let mut parts = version.split('.');
    let values = std::array::from_fn(|_| {
        parts
            .next()
            .filter(|part| part.bytes().all(|byte| byte.is_ascii_digit()))?
            .parse::<u64>()
            .ok()
    });
    let [Some(major), Some(minor), Some(patch)] = values else {
        return None;
    };
    parts.next().is_none().then_some([major, minor, patch])
}

pub(super) fn inspect_manifest(
    client: &mut Client,
    path: &Path,
    installed: String,
    bundled: &str,
) -> Option<Version> {
    let Some((installed_parts, bundled_parts)) = components(&installed).zip(components(bundled))
    else {
        client.add(
            Component::Plugin,
            State::Invalid,
            Some(path),
            "Could not compare the installed and bundled Mash plugin versions.",
            Some(REINSTALL_PLUGIN),
        );
        return None;
    };
    let order = installed_parts.cmp(&bundled_parts);
    let (state, message, remedy) = match order {
        Ordering::Less => (
            State::Outdated,
            format!(
                "Installed Mash plugin {installed} is older than {bundled}, the version bundled with this CLI."
            ),
            Some(UPDATE_PLUGIN),
        ),
        Ordering::Equal => (
            State::Present,
            format!("Installed Mash plugin {installed} matches the version bundled with this CLI."),
            None,
        ),
        Ordering::Greater => (
            State::Present,
            format!(
                "Installed Mash plugin {installed} is newer than {bundled}, the version bundled with this CLI."
            ),
            None,
        ),
    };
    client.add(Component::Plugin, state, Some(path), &message, remedy);
    Some(Version { installed, order })
}

pub(super) fn inspect_stamp(
    client: &mut Client,
    path: &Path,
    version: Option<&Version>,
    expected: &Stamp,
) {
    let Ok(Some(stamp)) = read::<Stamp>(path) else {
        client.add(
            Component::Skill,
            State::Outdated,
            Some(path),
            "The Mash history skill's contract stamp is missing or unreadable.",
            Some(REINSTALL_PLUGIN),
        );
        return;
    };
    let Some(version) = version else {
        return;
    };
    if stamp.plugin_version != version.installed {
        client.add(
            Component::Skill,
            State::Invalid,
            Some(path),
            "The Mash history skill and the installed plugin manifest disagree on the plugin version.",
            Some(REINSTALL_PLUGIN),
        );
    } else if stamp.contract_fingerprint != expected.contract_fingerprint {
        let remedy = match version.order {
            Ordering::Less => UPDATE_PLUGIN,
            Ordering::Greater => UPDATE_CLI,
            Ordering::Equal => {
                "Install the Mash plugin paired with this CLI release, then restart the agent."
            }
        };
        client.add(
            Component::Skill,
            State::Outdated,
            Some(path),
            &format!(
                "Mash plugin {} declares a different tool contract from the plugin bundled with this CLI ({}).",
                version.installed, expected.plugin_version
            ),
            Some(remedy),
        );
    } else if version.order == Ordering::Greater {
        client.add(
            Component::Skill,
            State::Present,
            Some(path),
            "The newer Mash plugin declares the same tool contract as this CLI.",
            None,
        );
    }
}

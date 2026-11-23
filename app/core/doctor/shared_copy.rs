//! Whether this machine's history is reaching the shared index: published,
//! behind, refused (recorded by the watcher), or not checkable from here.
use super::{Check, Finding, State, shared_target};
use crate::config::ResolvedConfig;
use crate::core::models::StatusResponse;

pub(super) fn shared_copy(configuration: &ResolvedConfig, status: &StatusResponse) -> Finding {
    let Some(shared) = shared_target(configuration) else {
        return Finding::ok(
            Check::SharedCopy,
            "This machine does not publish to a shared index.",
        );
    };
    if let Some(finding) = refused(status.shared.as_ref(), &shared.token_env) {
        return finding;
    }
    if shared.token().is_none() {
        // The machine may be publishing fine; this process cannot tell.
        return Finding::found(
            Check::SharedCopy,
            State::Warning,
            "The shared copy was not checked: its token is not available to this process.",
        )
        .cause(format!(
            "`{}` is not set here. A watcher started by its launcher has it; a plain shell may not.",
            shared.token_env
        ))
        .repair(format!("Export `{}` in this environment, then run `mash doctor` again.", shared.token_env));
    }
    if let Some(error) = &status.shared_error {
        return Finding::found(
            Check::SharedCopy,
            State::Problem,
            format!("The shared index at {} did not answer.", shared.api_url),
        )
        .cause(error.clone());
    }
    let published = status.shared_index.as_ref().and_then(|index| {
        index
            .nodes
            .iter()
            .find(|node| node.node_id == shared.node_id)
    });
    let Some(published) = published else {
        return Finding::found(
            Check::SharedCopy,
            State::Warning,
            "This machine has published nothing to the shared index yet.",
        )
        .repair("mash index");
    };
    let current = published
        .source_silver
        .as_ref()
        .map(|silver| silver.revision)
        == status.silver.as_ref().map(|silver| silver.revision);
    if current {
        return Finding::ok(
            Check::SharedCopy,
            format!("The shared copy is current ({} chats).", published.chats),
        );
    }
    let finding = Finding::found(
        Check::SharedCopy,
        State::Warning,
        "The shared copy is behind this machine's index.",
    )
    .repair("mash index (publishes now and reports why if the server refuses)");
    match status.shared.as_ref().and_then(|shared| shared.error.as_deref()) {
        Some(error) => finding.cause(error.to_owned()),
        None => finding.cause("A publication is pending. The watcher publishes after each pass, and writes a `shared-publication-failed` line to its log when one is refused."),
    }
}

/// The watcher records each refusal beside the database, so a refusal is known
/// even in a process without the token, which could otherwise only say the
/// copy was not checked.
pub(super) fn refused(
    publication: Option<&crate::shared::Publication>,
    token_env: &str,
) -> Option<Finding> {
    let refusal = publication
        .filter(|publication| publication.is_pending())
        .and_then(|publication| publication.error.as_deref())
        .filter(|error| error.starts_with(crate::shared::publication::failure::REFUSED))?;
    Some(
        Finding::found(Check::SharedCopy, State::Problem, refusal.to_owned())
            .cause("The shared index is not getting this machine's new chats.")
            .repair(format!(
                "If it names {token_env}, start the watcher with this machine's credentials (its launcher loads them); otherwise fix what it names. `mash index` publishes now and says why if it is refused."
            )),
    )
}

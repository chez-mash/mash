//! Readiness for `status`: the local search service, the index, the agents
//! connected to it and the shared copy.
use super::{Status, json, line, safe, warning};

pub(super) fn coverage(
    out: &mut String,
    value: &crate::core::coverage::Report,
) -> Result<(), serde_json::Error> {
    line(out, "\nScoped coverage (retained data and recorded scans):");
    json(out, "  Requested scope", &value.scope)?;
    line(out, &format!("  Absence proven: {}", value.absence_proven));
    json(out, "  Limitations", &value.limitations)?;
    json(out, "  Unavailable nodes", &value.unavailable_nodes)?;
    for member in &value.members {
        line(out, &format!("  Node: {}", safe(&member.node_id)));
        json(out, "    Index", &member.index)?;
        match &member.retained {
            Some(rows) => json(out, "    Retained rows (message bounds in Unix ms)", rows)?,
            None => line(out, "    Scoped retained rows: unknown/unreported"),
        }
        if let Some(certificate) = &member.source_silver {
            json(out, "    Source silver", certificate)?;
        }
        match &member.providers {
            Some(scans) => {
                for scan in scans {
                    line(
                        out,
                        &format!(
                            "    Provider: {} · configured={} · rejected_sources={}",
                            scan.provider, scan.configured, scan.rejected_sources
                        ),
                    );
                    json(
                        out,
                        "      Recorded scan state (not historical completeness)",
                        &scan.state,
                    )?;
                }
            }
            None => line(out, "    Provider scan coverage: unknown/unreported"),
        }
    }
    Ok(())
}

pub(super) fn status(out: &mut String, value: &Status<'_>) -> Result<(), serde_json::Error> {
    line(
        out,
        &format!("Mash status — local search ready: {}", value.ready),
    );
    line(
        out,
        &format!(
            "Local Qdrant: {} at {}",
            super::plain(&serde_json::to_value(value.local_qdrant.state)?),
            safe(&value.local_qdrant.url)
        ),
    );
    warning(out, "local Qdrant", value.local_qdrant.message.as_deref());
    if let Some(watcher) = value
        .watcher
        .filter(|watcher| watcher.running == Some(false))
    {
        warning(
            out,
            "watcher",
            Some(&format!(
                "a watcher service is installed ({}) but no `mash index --watch` runs for this index, so history changes are not indexed automatically. Start it with its service manager (`launchctl kickstart gui/$(id -u)/dev.mash.index-watch` on macOS, `systemctl --user start mash-index-watch.service` on Linux) and check its log.",
                watcher.definition.display()
            )),
        );
    }
    json(out, "Artifacts", value.local_artifacts)?;
    agents(out, &serde_json::to_value(value.installation)?);
    json(out, "Query models", value.query_models)?;
    line(
        out,
        &format!(
            "Models installed: {} · dense support: {}",
            value.models_installed, value.semantic_support
        ),
    );
    line(
        out,
        &format!(
            "Retained: {} chats, {} messages · searchable: {} · embeddings: {}",
            value.chats, value.messages, value.searchable_messages, value.embeddings
        ),
    );
    json(out, "Mode", value.realized_mode)?;
    if let Some(action) = value.recommended_action {
        json(out, "Next action", action)?;
    }
    if let Some(silver) = value.silver {
        json(out, "Silver", silver)?;
    }
    line(out, "Readiness does not certify complete provider history.");
    warning(out, "shared status", value.shared_error);
    if let Some(publication) = &value.shared {
        json(out, "Shared publication", &publication.state)?;
        warning(out, "shared publication", publication.error);
    }
    if let Some(shared) = &value.shared_index {
        freshness(out, shared)?;
    }
    if let Some(value) = value.coverage {
        coverage(out, value)?;
    }
    Ok(())
}

pub(super) fn freshness(
    out: &mut String,
    shared: &crate::shared::FreshnessSummary<'_>,
) -> Result<(), serde_json::Error> {
    line(out, &format!("Shared group: {}", safe(shared.group_id)));
    if let Some(unpublished) = shared.unpublished_nodes {
        json(out, "Unpublished nodes", &unpublished)?;
    }
    for node in &shared.nodes {
        line(
            out,
            &format!(
                "  Node {} · revision={} · {} chats, {} messages · lexical_current={} · views_current={}",
                safe(node.node_id),
                node.revision,
                node.chats,
                node.messages,
                node.lexical_current,
                node.views_current
            ),
        );
        json(out, "    Last synced", &node.last_synced_at)?;
        json(out, "    Last updated", &node.last_updated_at)?;
        if let Some(artifacts) = node.artifacts {
            json(out, "    Artifacts", artifacts)?;
        }
        if let Some(silver) = node.source_silver {
            json(out, "    Source silver", silver)?;
        }
    }
    Ok(())
}

/// One line per connected agent, and what would repair the first that needs
/// it; `mash status --installation` has the findings.
fn agents(out: &mut String, installation: &serde_json::Value) {
    let clients = installation["clients"]
        .as_array()
        .map_or(&[][..], Vec::as_slice);
    let described = clients
        .iter()
        .map(|client| {
            let agent = client["agent"].as_str().unwrap_or("agent");
            let connection = client["connection"].as_str().unwrap_or("unknown");
            match client["attention"].as_u64().unwrap_or(0) {
                0 => format!("{agent} ({connection})"),
                attention => format!("{agent} ({connection}, {attention} to fix)"),
            }
        })
        .collect::<Vec<_>>();
    if !described.is_empty() {
        line(out, &format!("Agents: {}", safe(&described.join(" · "))));
    }
    warning(out, "agents", installation["next_action"].as_str());
}

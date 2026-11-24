//! What `init` tells the reader to do next: how to get past a Qdrant that is
//! not running, and what each agent still needs.
use super::{AgentOutcome, Step};

/// The way past a missing Qdrant, first: it is all a new user needs to read.
pub(super) const CONNECT_NOW: &str =
    "To connect your agents now and index later, run `mash init --no-index`.";

/// How to read history before Qdrant runs: indexing saves it first.
pub(super) const BROWSE_NOW: &str = "To read your chats before Qdrant runs, run `mash init --no-index`, then `mash index`: indexing saves your history before it needs Qdrant, and word search with `mash search`, `mash list-chats` and `mash view <message id>` read it without Qdrant.";

/// The same route after `--no-index` has saved the configuration.
const INDEX_SAVES_HISTORY: &str = "To read your chats now, run `mash index`: it saves your history before it needs Qdrant, and word search with `mash search`, `mash list-chats` and `mash view <message id>` read it without Qdrant.";

/// The `--no-index` step: index now, or start Qdrant first.
pub(super) fn index_later(observed: Option<crate::views::qdrant::ServiceStatus>) -> String {
    use crate::views::qdrant::Reachability;
    match observed {
        Some(observed) if observed.state == Reachability::Reachable => {
            "Index your history: mash index".to_owned()
        }
        Some(observed) => {
            let unreachable = (observed.state == Reachability::Unreachable)
                .then(|| {
                    crate::views::qdrant::unavailable_message(
                        &observed.url,
                        Some(INDEX_SAVES_HISTORY),
                        "run `mash index` to make search current",
                    )
                })
                .flatten();
            unreachable.unwrap_or_else(|| {
                format!(
                    "Index your history with `mash index` once Qdrant answers at {}; indexing and ranked search need it. {}",
                    observed.url,
                    observed.message.unwrap_or_default()
                )
            })
        }
        None => "Index your history: mash index".to_owned(),
    }
}

/// What each agent still needs after registration, and whether to restart.
pub(super) fn agent_next_steps(agents: &[AgentOutcome]) -> Vec<String> {
    let mut steps = Vec::new();
    for agent in agents {
        let name = agent.agent.as_str();
        if let Step::Failed { command, .. } = &agent.mcp {
            steps.push(format!(
                "{name}: the Mash MCP server was not registered, so this agent cannot search history yet; fix the error above, then rerun `mash init` or run `{command}`."
            ));
        } else if let Step::Stale { stale, command, .. } = &agent.mcp {
            steps.push(format!(
                "{name}: the Mash MCP registration runs {stale}, which is missing or is not this Mash binary, and could not be replaced (see the error above). Run `{command}`, then restart the agent."
            ));
        } else if let Step::Failed { command, .. } = &agent.plugin {
            steps.push(format!(
                "{name}: the Mash MCP server is registered, but the mash@mash plugin (history skill and session hooks) was not installed; when online, rerun `mash init` or run `{command}`."
            ));
        }
    }
    if agents
        .iter()
        .any(|agent| !matches!(agent.mcp, Step::Failed { .. } | Step::Stale { .. }))
    {
        steps.push("Restart your agents so they load Mash.".to_owned());
    }
    steps
}

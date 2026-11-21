//! A short report for the interactive setup wizard.
use std::fmt::Write as _;

use super::{InitResponse, Response, Step};
use crate::core::output::text::safe;

const SEMANTIC_QUERY: &str = "Word search is the default. To search by meaning after indexing, run `mash search 'your question' --intent related-discussion`.";

pub(super) const fn search_guidance(editing: bool, semantic: bool) -> Option<&'static str> {
    if editing {
        None
    } else if semantic {
        Some(SEMANTIC_QUERY)
    } else {
        Some(
            "Search is set to word search (`exact-mention`). For search by meaning, install the models and update your configuration. See https://getmash.io/indexing/#enable-semantic-search",
        )
    }
}

impl InitResponse {
    #[must_use]
    pub fn terminal_summary(&self, dashboard_available: bool) -> String {
        let mut output = String::new();
        let mut searchable = false;
        match (&self.index_error, self.index.as_deref()) {
            (Some(error), _) => {
                let _ = writeln!(
                    output,
                    "Settings saved. Indexing stopped. {}",
                    safe(&error.error.message)
                );
                output.push_str("After fixing this, run `mash index` to finish.\n");
            }
            (_, Some(Response::Index(index))) if index.needs_retry() => {
                output.push_str("Settings saved. Indexing needs attention. Run `mash doctor`.\n");
            }
            (_, Some(Response::Index(index))) if index.searchable_messages == 0 => {
                output.push_str(
                    "Indexing finished. No messages are searchable yet. Run `mash doctor` to check the history folders.\n",
                );
            }
            (_, Some(Response::Index(index))) => {
                let _ = writeln!(
                    output,
                    "Search is ready. Indexed chats: {}. Searchable messages: {}.",
                    index.indexed_chats, index.searchable_messages
                );
                searchable = true;
            }
            _ => {
                let action = if self.config.written { "Saved" } else { "Kept" };
                let _ = writeln!(
                    output,
                    "{action} settings in {}. Indexing was skipped.",
                    safe(&self.config.path.to_string_lossy())
                );
                if let Some(next) = self.next.last() {
                    let _ = writeln!(output, "{}", safe(next));
                }
            }
        }
        let connected = self
            .agents
            .iter()
            .filter(|agent| !matches!(agent.mcp, Step::Failed { .. } | Step::Stale { .. }))
            .map(|agent| agent.agent.display_name())
            .collect::<Vec<_>>();
        if !connected.is_empty() {
            let _ = writeln!(
                output,
                "Connected {}. Restart them so they can search your history.",
                connected.join(", ")
            );
        }
        if self.agents.iter().any(|agent| {
            matches!(agent.mcp, Step::Failed { .. } | Step::Stale { .. })
                || matches!(agent.plugin, Step::Failed { .. })
        }) {
            output.push_str(
                "An agent connection needs attention. Run `mash status --installation`.\n",
            );
        }
        if let Some(memory) = &self.memory {
            let _ = writeln!(output, "{}", safe(&memory.message));
        }
        if searchable {
            output.push_str(if dashboard_available {
                "Start the browser dashboard with `mash dash` and open the address it prints.\n"
            } else {
                "Search in the terminal with `mash search migration`. Replace `migration` with a word from your chat.\n"
            });
            if self.agents.is_empty() {
                output.push_str("You can connect an agent later so it can search Mash for you. See https://getmash.io/agents/\n");
            }
            if self.config.search == Some("semantic") {
                let _ = writeln!(output, "{SEMANTIC_QUERY}");
            }
        }
        output
    }
}

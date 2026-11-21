//! A short report for the interactive setup wizard.
use std::fmt::Write as _;

use super::{InitResponse, Response, Step};
use crate::core::output::text::safe;

impl InitResponse {
    #[must_use]
    pub fn terminal_summary(&self) -> String {
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
                "Connected {}. Restart to load Mash.",
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
            output.push_str("Open your chats with `mash dash`.\n");
        }
        output
    }
}

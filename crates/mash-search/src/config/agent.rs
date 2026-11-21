//! The agent CLIs Mash can register itself with, and the commands that do it.
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[schemars(rename = "SetupAgent")]
pub enum Agent {
    Codex,
    ClaudeCode,
    // GitHub Copilot CLI. A doc comment here would turn the generated
    // schema's flat enum into a `oneOf`.
    Copilot,
}

impl Agent {
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::ClaudeCode => "Claude Code",
            Self::Copilot => "GitHub Copilot",
        }
    }

    /// The `--agent` value and JSON name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude-code",
            Self::Copilot => "copilot",
        }
    }

    /// The agent's own command-line program.
    pub const fn program(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude",
            Self::Copilot => "copilot",
        }
    }

    /// The agent CLI command that removes Mash's user-level MCP registration.
    pub const fn remove_mcp(self) -> &'static str {
        match self {
            Self::Codex => "codex mcp remove mash",
            Self::ClaudeCode => "claude mcp remove --scope user mash",
            // Copilot's `mcp` commands edit user settings only; they take no scope.
            Self::Copilot => "copilot mcp remove mash",
        }
    }

    /// The agent CLI command that registers `executable` as Mash's MCP server.
    pub fn add_mcp(self, executable: &str) -> String {
        match self {
            Self::Codex => format!("codex mcp add mash -- {executable} mcp"),
            Self::ClaudeCode => format!("claude mcp add --scope user mash -- {executable} mcp"),
            Self::Copilot => format!("copilot mcp add mash -- {executable} mcp"),
        }
    }

    /// The online commands that install and enable the mash@mash plugin.
    pub const fn install_plugin(self) -> &'static str {
        match self {
            Self::ClaudeCode => {
                "claude plugin marketplace add mashium/mash && claude plugin install mash@mash"
            }
            Self::Codex => {
                "codex plugin marketplace add mashium/mash && codex plugin add mash@mash"
            }
            Self::Copilot => {
                "copilot plugin marketplace add mashium/mash && copilot plugin install mash@mash"
            }
        }
    }

    /// The agent CLI command that removes the mash@mash plugin.
    pub const fn remove_plugin(self) -> &'static str {
        match self {
            Self::Codex => "codex plugin remove mash@mash",
            Self::ClaudeCode => "claude plugin uninstall mash@mash",
            Self::Copilot => "copilot plugin uninstall mash@mash",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Agent;

    #[test]
    fn agent_copilot_commands_are_user_level_and_leave_the_others_unchanged() {
        let agent = Agent::Copilot;
        assert_eq!(agent.as_str(), "copilot");
        assert_eq!(agent.program(), "copilot");
        assert_eq!(serde_json::to_string(&agent).unwrap(), "\"copilot\"");
        assert_eq!(
            agent.add_mcp("/bin/mash"),
            "copilot mcp add mash -- /bin/mash mcp"
        );
        assert_eq!(agent.remove_mcp(), "copilot mcp remove mash");
        assert_eq!(
            agent.install_plugin(),
            "copilot plugin marketplace add mashium/mash && copilot plugin install mash@mash"
        );
        assert_eq!(agent.remove_plugin(), "copilot plugin uninstall mash@mash");
        assert_eq!(
            Agent::ClaudeCode.remove_mcp(),
            "claude mcp remove --scope user mash"
        );
        assert_eq!(
            Agent::Codex.remove_plugin(),
            "codex plugin remove mash@mash"
        );
    }
}

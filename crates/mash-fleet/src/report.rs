use rmcp::model::{CallToolResult, ProgressNotificationParam};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    Running,
    Succeeded,
    Failed,
    Skipped,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Report {
    pub state: State,
    #[serde(default)]
    pub progress: Option<ProgressNotificationParam>,
    #[serde(default)]
    pub error: Option<String>,
    /// What a successful check established, in one line a reader can be shown.
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub results: Vec<CallToolResult>,
    /// What a doctor examination found on the machine, problems included.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<Finding>,
}

/// One thing `mash doctor` examined on a machine. The check is kept as the
/// machine named it, so a newer release's checks are still reported.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Finding {
    pub check: String,
    pub state: Health,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repair: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Health {
    Ok,
    Warning,
    Problem,
}

impl Default for Report {
    fn default() -> Self {
        Self {
            state: State::Running,
            progress: None,
            error: None,
            summary: None,
            results: vec![],
            findings: vec![],
        }
    }
}

//! Mash's local usage log: one row for every tool or command call, with its
//! arguments, outcome and timing, in `usage.sqlite3` beside the index.
//!
//! Each surface records its own calls once, at its boundary: the CLI for each
//! command it runs, the MCP server for each tool call, the dashboard for each
//! request that searches or reads history. Recording never slows or fails a
//! call: rows go through a bounded channel to one writer thread, and a full
//! channel drops the row and counts the drop. The log stays on this machine;
//! it is never published, exported or synced, and its rows are kept until the
//! person clears them.
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::AppError;
use crate::core::Response;

mod references;
mod store;
mod writer;

pub use store::{clear, enabled, set_enabled};

/// The file name of the log, beside the index it records calls against.
pub const FILE_NAME: &str = "usage.sqlite3";

/// Where a call came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Surface {
    Cli,
    Mcp,
    Dashboard,
}

impl Surface {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Mcp => "mcp",
            Self::Dashboard => "dashboard",
        }
    }
}

/// How a call ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    Ok,
    Error,
    /// The call answered, but asked to be retried (an index still catching up).
    Incomplete,
}

impl Outcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Incomplete => "incomplete",
        }
    }
}

/// One recorded call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Call {
    /// When the call started, in milliseconds since the Unix epoch.
    pub at_ms: i64,
    pub surface: Surface,
    /// The program that made the call: an MCP client's name, the agent a
    /// command ran under, or a browser family.
    pub client: Option<String>,
    /// The MCP client's version, from its handshake.
    pub client_version: Option<String>,
    /// The tool, command or dashboard route.
    pub tool: String,
    /// The call's arguments as given, with recognizable credentials masked.
    pub arguments: serde_json::Value,
    pub outcome: Outcome,
    pub error_kind: Option<String>,
    pub error_message: Option<String>,
    pub duration_ms: u64,
    pub result_count: Option<u64>,
    pub response_bytes: Option<u64>,
    /// Groups the calls of one MCP connection.
    pub session: Option<String>,
    pub process_id: u32,
    /// The Mash build that served the call.
    pub build: String,
    /// The working directory, which names the project a command ran in.
    pub cwd: Option<String>,
    /// The response, when the surface has it. The writer reads references to
    /// what the call returned from it, off the caller's path, and never
    /// stores the response itself.
    #[serde(skip)]
    pub response: Option<Returned>,
}

/// A call's response, as the surface holds it.
#[derive(Debug, Clone, PartialEq)]
pub enum Returned {
    /// JSON text, as the CLI printed it.
    Text(String),
    /// Structured JSON, as an MCP tool result carries it.
    Json(serde_json::Value),
}

/// Marks when a call started; [`Started::call`] turns it into a row.
#[derive(Debug, Clone, Copy)]
pub struct Started {
    at: SystemTime,
    clock: Instant,
}

impl Started {
    #[must_use]
    pub fn now() -> Self {
        Self {
            at: SystemTime::now(),
            clock: Instant::now(),
        }
    }

    /// A successful call of `tool` with `arguments`, timed from this start.
    #[must_use]
    pub fn call(self, surface: Surface, tool: &str, arguments: serde_json::Value) -> Call {
        let at_ms = self.at.duration_since(UNIX_EPOCH).map_or(0, |since| {
            i64::try_from(since.as_millis()).unwrap_or(i64::MAX)
        });
        Call {
            at_ms,
            surface,
            client: None,
            client_version: None,
            tool: tool.to_owned(),
            arguments,
            outcome: Outcome::Ok,
            error_kind: None,
            error_message: None,
            duration_ms: u64::try_from(self.clock.elapsed().as_millis()).unwrap_or(u64::MAX),
            result_count: None,
            response_bytes: None,
            session: None,
            process_id: std::process::id(),
            build: crate::BUILD_REVISION.to_owned(),
            cwd: std::env::current_dir()
                .ok()
                .map(|directory| directory.display().to_string()),
            response: None,
        }
    }
}

impl Call {
    /// Mark the call failed with `error`.
    pub fn fail(&mut self, error: &AppError) {
        self.outcome = Outcome::Error;
        self.error_kind = serde_json::to_value(error.error.kind)
            .ok()
            .and_then(|kind| kind.as_str().map(str::to_owned));
        self.error_message = Some(error.error.message.clone());
    }

    /// Take the outcome and result count from a core response.
    pub fn answered(&mut self, response: &Response) {
        if let Some(error) = response.failure() {
            self.fail(error);
        } else if response.needs_retry() {
            self.outcome = Outcome::Incomplete;
        }
        self.result_count = result_count(response);
    }
}

/// How many results a response carries, for the responses that list some.
#[must_use]
pub fn result_count(response: &Response) -> Option<u64> {
    let length = match response {
        Response::Search(search) => match &search.results {
            crate::search_results::Results::Chats(chats) => chats.len(),
            crate::search_results::Results::Messages(messages) => messages.len(),
        },
        Response::ListChats(listing) => listing.chats.len(),
        Response::Records(page) => page.records.len(),
        _ => return None,
    };
    u64::try_from(length).ok()
}

/// The log that sits beside `database`.
#[must_use]
pub fn path_for(database: &Path) -> PathBuf {
    database.with_file_name(FILE_NAME)
}

/// Queue `call` for the log beside `database`. Never blocks and never fails:
/// a call that cannot be queued is dropped and counted. `MASH_USAGE=0` turns
/// recording off for the whole process (the per-log setting is
/// [`set_enabled`]).
pub fn record(database: &Path, mut call: Call) {
    if off_by_environment() {
        return;
    }
    scrub(&mut call);
    writer::submit(path_for(database), call);
}

/// Wait up to `timeout` for every queued call to be written. Processes call
/// this before they exit; `false` means the writer did not finish in time.
pub fn flush(timeout: Duration) -> bool {
    writer::flush(timeout)
}

fn off_by_environment() -> bool {
    std::env::var_os("MASH_USAGE").is_some_and(|value| value == "0" || value == "off")
}

/// The agent a command runs under, when its environment says so. `MASH_CLIENT`
/// names a caller explicitly (Raycast sets it); Claude Code sets `CLAUDECODE`
/// for the commands it runs.
#[must_use]
pub fn client_from_environment() -> Option<String> {
    if let Some(client) = std::env::var_os("MASH_CLIENT") {
        let client = client.to_string_lossy().trim().to_owned();
        if !client.is_empty() {
            return Some(client);
        }
    }
    (std::env::var_os("CLAUDECODE").as_deref() == Some(std::ffi::OsStr::new("1")))
        .then(|| "claude-code".to_owned())
}

/// Credentials never reach the log, even inside arguments: recognizable keys
/// and tokens are masked as everywhere else Mash shows text, and join strings
/// and admin keys, which carry a shared server credential, are replaced.
fn scrub(call: &mut Call) {
    scrub_value(&mut call.arguments);
    if let Some(message) = &mut call.error_message {
        *message = scrub_text(message);
    }
}

fn scrub_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(text) => *text = scrub_text(text),
        serde_json::Value::Array(items) => items.iter_mut().for_each(scrub_value),
        serde_json::Value::Object(fields) => fields.values_mut().for_each(scrub_value),
        _ => {}
    }
}

fn scrub_text(text: &str) -> String {
    const SHARED_CREDENTIALS: [&str; 2] = ["mash-join.", "mash-admin."];
    let masked = crate::redaction::Policy::default().text(text);
    masked
        .split_inclusive(char::is_whitespace)
        .map(|word| {
            let start = word
                .find(SHARED_CREDENTIALS[0])
                .or_else(|| word.find(SHARED_CREDENTIALS[1]));
            match start {
                Some(start) => {
                    let end = word.trim_end().len();
                    format!("{}[REDACTED]{}", &word[..start], &word[end..])
                }
                None => word.to_owned(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;

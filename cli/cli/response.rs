//! What a command answered, which `lib.rs` prints or acts on.
use super::{AppError, core, usage};
use serde::Serialize;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum Response {
    Operation(Box<core::Response>),
    Setup(Box<core::Response>),
    Help {
        help: String,
    },
    Usage(usage::Answer),
    /// The release and the silver schema it opens; deployment compares the
    /// latter with an existing database's `user_version` before replacing it.
    Version {
        version: &'static str,
        /// The commit this binary was built from: two copies of one version
        /// can still be different builds.
        build: &'static str,
        silver_schema: i64,
        silver_schema_min: i64,
        /// How `record` evidence may name its source. The plugin hooks read
        /// this before telling an agent to cite its own session.
        record_citations: [&'static str; 2],
        /// The agents whose running session a `session_id` can name before it
        /// is indexed. The hooks read this before telling one of them to.
        session_citations: [&'static str; 3],
    },
    WatchStopped {
        watching: bool,
    },
    /// The watcher stopped because its executable was replaced; the caller
    /// starts the replacement in this process once the runtime is gone.
    #[serde(skip)]
    WatchReplaced {
        reason: AppError,
        executable: PathBuf,
    },
    /// `mcp` or `dash` ran until it was stopped.
    ServerStopped,
    /// `search --stream` printed its updates as they came; `results` is how
    /// many the last one listed, for the usage log.
    #[serde(skip)]
    Streamed {
        results: u64,
    },
    InitCancelled {
        cancelled: bool,
    },
}

/// Whether the command line asks for JSON at a terminal: `--json` before any
/// `--`, after which an argument is a value such as a search query.
pub(crate) fn asks_for_json(arguments: &[OsString]) -> bool {
    arguments
        .iter()
        .skip(1)
        .take_while(|argument| *argument != "--")
        .any(|argument| argument == "--json")
}

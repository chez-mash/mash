// Typed async search futures include transport tracing and concurrent retrieval.
#![recursion_limit = "256"]

// Silver canonical history and the vocabulary used by native adapters.
pub mod openapi;
pub use mash_model::{message_context, provider, role, tools};
pub mod silver;
pub use mash_storage::silver_query;

// View materialization, retrieval models, policies and query-oriented views.
pub use mash_storage::chat_catalog;
pub use mash_storage::search_filter;
use mash_storage::search_query;
pub use mash_storage::search_results;
pub use mash_storage::subagents;
/// The models and embeddings, with the relevance harness that drives the core.
pub mod semantic {
    pub use mash_search::semantic::*;
    #[cfg(any(test, feature = "test-support"))]
    pub mod evaluation;
}
/// Local and shared views; the Qdrant tests that run against the shared
/// server's store live here.
pub mod views {
    pub use mash_search::views::*;
    #[cfg(test)]
    pub mod qdrant {
        use crate::AppError;
        use crate::views::scope::Scope;
        pub use mash_search::views::qdrant::*;
        use serde::{Deserialize, Serialize};
        mod tests;
    }
}

// Application composition and infrastructure supporting both layers.
use mash_model::{Action, AppError, ErrorKind};
pub use mash_model::{attachments, database_error, platform, progress, redaction, scratch};
use mash_search::search_ranking;
pub use mash_search::{blocking, config};
pub mod core;
use mash_storage::export;
/// Registers the silver text codec on a raw connection, as tests need to read silver directly.
#[doc(hidden)]
pub use storage::packed::register as register_silver_codec;
/// Tracing and retrieval measurements; the tests that index through the core live here.
pub mod observability {
    pub use mash_search::observability::*;
    #[cfg(test)]
    mod tests;
}
#[cfg(test)]
mod progress_tests;
#[cfg(test)]
mod storage_tests;
pub use mash_storage::records;
pub mod remote;
pub mod runtime;
pub(crate) mod service;
#[cfg(test)]
use mash_model::test_support;
pub use mash_storage::storage;
/// The shared index; its tests that search through the core live here.
pub mod shared {
    pub use mash_sync::shared::*;
    #[cfg(test)]
    mod diagnostics_core_tests;
    #[cfg(test)]
    mod exact_core_tests;
}
/// The local log of every tool and command call; see `usage`.
pub mod usage;
pub mod watch;

use std::io::{self, Write};

use serde::Serialize;

static REVISION: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();

/// The commit the running program was built from, or `unknown` before
/// [`set_build_revision`] has been called, as in this crate's own tests.
pub fn build_revision() -> &'static str {
    REVISION.get().copied().unwrap_or("unknown")
}

/// Records the commit the running program was built from. The first call
/// wins; `mash` and `mash-server` make it before anything else.
///
/// The commit is compiled into the package that builds those programs, not
/// into this library. It differs for every commit, and a value compiled in
/// here would recompile this crate and every crate built on it for each one,
/// which is most of a build: only the programs' own package needs to change.
pub fn set_build_revision(revision: &'static str) {
    let _ = REVISION.set(revision);
}

/// Internal code for a closed output stream; never reported as an exit status.
pub const BROKEN_PIPE: i32 = -1;

pub fn write_json<T: Serialize>(writer: &mut dyn Write, value: &T) -> Result<(), AppError> {
    let closed = |mut error: AppError, kind: Option<io::ErrorKind>| {
        if kind == Some(io::ErrorKind::BrokenPipe) {
            error.code = BROKEN_PIPE;
        }
        error
    };
    serde_json::to_writer(&mut *writer, value).map_err(|error| {
        closed(
            AppError::internal(format!("failed to serialize response: {error}")),
            error.io_error_kind(),
        )
    })?;
    writeln!(writer).map_err(|error| {
        closed(
            AppError::internal(format!("failed to write response: {error}")),
            Some(error.kind()),
        )
    })
}

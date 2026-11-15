//! Explicit placement of contextual reranking. Local reranking is the default
//! and the offline path; a remote worker is opt-in.
use crate::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(tag = "backend", rename_all = "lowercase", deny_unknown_fields)]
pub enum Reranking {
    Local {
        /// Experimental. On Apple Silicon, rerank through a Core ML worker
        /// that Mash starts when a search needs it and that exits when idle.
        /// Searches use the CPU until it is ready. Off unless set; ignored
        /// on other machines.
        // Left out of status output while off, as before the setting existed.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        accelerated: bool,
    },
    Remote {
        url: String,
        token_env: String,
        /// How long a search waits for the worker before it reranks on the
        /// local CPU instead. A cold worker takes far longer than this.
        #[serde(default = "default_timeout")]
        timeout_milliseconds: u32,
    },
}

impl Default for Reranking {
    fn default() -> Self {
        Self::Local { accelerated: false }
    }
}

const fn default_timeout() -> u32 {
    1500
}

/// A search should never wait much longer for a remote worker than the local
/// CPU reranker takes to answer it.
const TIMEOUT_MILLISECONDS: std::ops::RangeInclusive<u32> = 100..=10_000;

impl Reranking {
    pub fn validate(&self) -> Result<(), AppError> {
        let Self::Remote {
            url,
            token_env,
            timeout_milliseconds,
        } = self
        else {
            return Ok(());
        };
        super::embeddings::validate_service(url, token_env, "reranking")?;
        if !TIMEOUT_MILLISECONDS.contains(timeout_milliseconds) {
            return Err(AppError::configuration(
                "reranking timeout_milliseconds must be between 100 and 10000",
            ));
        }
        Ok(())
    }
}

//! How a failed request to Qdrant is described: unreachable (nothing answered
//! the connection) or slow (it connected, then did not answer in time), with
//! the repair for each. Both are the same retryable error kind.
use super::Client;
use crate::AppError;

const START_GUIDE: &str = "https://getmash.io/installation/#start-the-local-search-service";

/// How to start Qdrant, readable offline. A plain-HTTP endpoint on this
/// machine is one `mash qdrant install` can provide; the default one also
/// gets the documented commands for a Qdrant the user runs. Any other
/// endpoint is the user's own service.
pub(crate) fn start_instruction(url: &reqwest::Url) -> &'static str {
    let local = url.scheme() == "http"
        && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
    match (local, url.port_or_known_default() == Some(6333)) {
        (true, true) => {
            "Install and start it with `mash qdrant install`. If you run Qdrant yourself, start that instead: `docker start mash-local-qdrant` for the documented container, or `QDRANT__SERVICE__HOST=127.0.0.1 ./qdrant` in a downloaded Qdrant 1.20 release folder."
        }
        (true, false) => {
            "Run `mash qdrant install` to install and start it on the configured ports. Start the configured Qdrant service instead if you run your own, and check its HTTP port."
        }
        (false, _) => "Start the configured Qdrant service and check its HTTP port.",
    }
}

/// One sequence for an unreachable Qdrant: what failed, `note` (if any), how
/// to start it, the single next step `then`, and the guide.
pub(crate) fn unavailable_message(url: &reqwest::Url, note: Option<&str>, then: &str) -> String {
    let note = note.map_or_default(|note| format!(" {note}"));
    format!(
        "Could not reach Qdrant at {url}; indexing and every search need it.{note} {} Then {then}. Guide: {START_GUIDE}",
        start_instruction(url)
    )
}

impl Client {
    /// A request that failed. One that connected but timed out reached a
    /// Qdrant too busy or stuck to answer, which "could not reach" would
    /// misdescribe; the error kind and retry are the same either way.
    pub(super) fn failed(&self, error: &reqwest::Error) -> AppError {
        if error.is_timeout() && !error.is_connect() {
            AppError::new(
                11,
                crate::ErrorKind::QdrantUnavailable,
                format!(
                    "Qdrant at {} is running but did not answer in time; it may be busy or stuck. Retry, and restart Qdrant if it keeps happening. Guide: {START_GUIDE}",
                    self.url
                ),
                true,
            )
        } else {
            self.unavailable()
        }
    }

    pub(crate) fn unavailable(&self) -> AppError {
        // endpoint() accepts only an origin: no credentials, paths, or queries.
        // Never include reqwest's error or the upstream response body.
        AppError::new(
            11,
            crate::ErrorKind::QdrantUnavailable,
            unavailable_message(&self.url, None, "retry"),
            true,
        )
    }
}

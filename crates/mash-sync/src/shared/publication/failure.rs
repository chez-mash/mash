//! What a background publisher says when the shared index refuses it.
//!
//! The watcher publishes after every pass with no caller to hand an error to,
//! so a refused publication was retried for hours while `mash status` and
//! `mash doctor`, running as other processes, could only say "pending". Each
//! refusal is therefore logged and recorded beside the database, with when the
//! refusals began; a publication that succeeds removes the record. Another
//! process holding the publication is not a failure and records nothing.
use super::SHARED_PENDING;
use crate::shared::Publication;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// How status begins a pending publication's error when the error is a
/// recorded refusal, so a reader such as `mash doctor` can tell it apart.
pub const REFUSED: &str = "The watcher's publications have been refused";

/// The last refusal, and when refusals began.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recorded {
    pub error: String,
    /// Unix milliseconds of the first refusal since the last success.
    pub since_ms: i64,
    /// Unix milliseconds of the latest refusal.
    pub last_ms: i64,
}

impl Recorded {
    /// The refusal as status reports it, saying how long it has lasted.
    pub fn describe(&self, now_ms: i64) -> String {
        let minutes = (now_ms - self.since_ms).max(0) / 60_000;
        let lasted = match minutes {
            0 => "since just now".to_owned(),
            1..=119 => format!("for {minutes} minutes"),
            _ => format!("for {} hours", minutes / 60),
        };
        format!(
            "The watcher's publications have been refused {lasted}: {}",
            self.error
        )
    }
}

fn path(database: &Path) -> PathBuf {
    let mut name = database.as_os_str().to_owned();
    name.push(".publication-failure.json");
    PathBuf::from(name)
}

pub(super) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
        })
}

/// The recorded refusal for this database, if its last publication failed.
/// An unreadable record is treated as none: it only adds an explanation.
pub fn recorded(database: &Path) -> Option<Recorded> {
    let text = std::fs::read_to_string(path(database)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Log and record a background publication's outcome: a refusal is written
/// with when refusals began, and a success removes the record.
pub(super) fn note(database: &Path, published: &Publication) {
    let refusal = published
        .error
        .as_deref()
        .filter(|error| published.is_pending() && !error.contains(SHARED_PENDING));
    let Some(error) = refusal else {
        if !published.is_pending() {
            let _ = std::fs::remove_file(path(database));
        }
        return;
    };
    log(published);
    let now = now_ms();
    let since_ms = recorded(database).map_or(now, |previous| previous.since_ms);
    let record = Recorded {
        error: error.to_owned(),
        since_ms,
        last_ms: now,
    };
    // Written beside and renamed over, so a reader never sees half a record.
    let target = path(database);
    let incoming = target.with_extension("json.incoming");
    if let Ok(text) = serde_json::to_vec(&record)
        && std::fs::write(&incoming, text).is_ok()
    {
        let _ = std::fs::rename(&incoming, &target);
    }
}

#[derive(Serialize)]
struct BackgroundFailure<'a> {
    event: &'static str,
    #[serde(flatten)]
    publication: crate::shared::output::PublicationSummary<'a>,
}

/// Tracing is off unless asked for, so the refusal also goes to stderr as one
/// JSON line, which the watcher's service log keeps.
fn log(published: &Publication) {
    let line = BackgroundFailure {
        event: "shared-publication-failed",
        publication: published.summary(),
    };
    let mut stderr = std::io::stderr().lock();
    if serde_json::to_writer(&mut stderr, &line).is_ok() {
        let _ = std::io::Write::write_all(&mut stderr, b"\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refused(error: &str) -> Publication {
        let mut publication = Publication::pending();
        publication.error = Some(error.to_owned());
        publication
    }

    #[test]
    fn a_refusal_is_recorded_from_its_first_occurrence_and_cleared_by_success() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("mash.sqlite3");
        note(
            &database,
            &refused("set MASH_NODE_TOKEN to this node's API token"),
        );
        let first = recorded(&database).unwrap();
        assert_eq!(first.error, "set MASH_NODE_TOKEN to this node's API token");
        note(&database, &refused("the server refused the token"));
        let second = recorded(&database).unwrap();
        assert_eq!(second.since_ms, first.since_ms, "refusals keep their start");
        assert_eq!(second.error, "the server refused the token");
        note(&database, &Publication::published());
        assert!(recorded(&database).is_none());
    }

    #[test]
    fn another_publisher_holding_the_database_is_not_a_refusal() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("mash.sqlite3");
        note(&database, &refused(SHARED_PENDING));
        assert!(recorded(&database).is_none());
    }

    #[test]
    fn the_description_says_how_long_refusals_have_lasted() {
        let record = Recorded {
            error: "no token".into(),
            since_ms: 0,
            last_ms: 0,
        };
        assert!(record.describe(30_000).contains("since just now"));
        assert!(record.describe(45 * 60_000).contains("for 45 minutes"));
        assert!(
            record
                .describe(5 * 3_600_000)
                .contains("for 5 hours: no token")
        );
    }
}

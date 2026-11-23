//! The usage log's SQLite file: schema, writes, the on/off setting and
//! clearing. The file is private to its owner, uses WAL so a reader (any
//! SQLite tool) never blocks the writer, and has its own `user_version`,
//! independent of the index.
use std::path::Path;

use super::Call;
use crate::AppError;
use rusqlite::{Connection, OptionalExtension, params};

const SCHEMA_VERSION: i64 = 1;

/// The tables are internal and may change between versions. The `calls` and
/// `call_results` views are the documented surface people query (see
/// `docs/usage-log.md`): their columns keep their names and meaning across
/// upgrades, and a migration that changes the tables recreates the views to
/// match.
const SCHEMA: &str = "
CREATE TABLE call_rows (
    id INTEGER PRIMARY KEY,
    at_ms INTEGER NOT NULL,
    surface TEXT NOT NULL,
    client TEXT,
    client_version TEXT,
    tool TEXT NOT NULL,
    arguments TEXT NOT NULL,
    outcome TEXT NOT NULL,
    error_kind TEXT,
    error_message TEXT,
    duration_ms INTEGER NOT NULL,
    result_count INTEGER,
    response_bytes INTEGER,
    results_total INTEGER,
    session TEXT,
    process_id INTEGER NOT NULL,
    build TEXT NOT NULL,
    cwd TEXT
);
CREATE INDEX call_rows_at ON call_rows (at_ms);
CREATE INDEX call_rows_tool ON call_rows (tool, at_ms);
CREATE TABLE result_rows (
    call_id INTEGER NOT NULL REFERENCES call_rows (id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    kind TEXT NOT NULL,
    record_kind TEXT,
    item_id TEXT NOT NULL,
    node TEXT,
    PRIMARY KEY (call_id, position)
);
CREATE INDEX result_rows_item ON result_rows (kind, record_kind);
CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE counters (key TEXT PRIMARY KEY, value INTEGER NOT NULL);
CREATE VIEW calls AS SELECT
    id,
    strftime('%Y-%m-%dT%H:%M:%fZ', at_ms / 1000.0, 'unixepoch') AS at,
    at_ms,
    surface,
    client,
    client_version,
    tool,
    arguments,
    outcome,
    error_kind,
    error_message,
    duration_ms,
    result_count,
    response_bytes,
    results_total,
    results_total > (SELECT COUNT(*) FROM result_rows WHERE call_id = call_rows.id) AS results_truncated,
    session,
    process_id AS pid,
    build,
    cwd AS project
FROM call_rows;
CREATE VIEW call_results AS SELECT
    call_id,
    position,
    kind,
    record_kind,
    item_id AS id,
    node
FROM result_rows;
";

fn io(error: &rusqlite::Error) -> AppError {
    AppError::internal(format!("usage log: {error}"))
}

/// Open the log at `path`, creating it private to its owner.
pub(super) fn open(path: &Path) -> Result<Connection, AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::io)?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.create(true).truncate(false).append(true);
    crate::platform::private_file(&mut options);
    drop(options.open(path).map_err(AppError::io)?);
    let connection = Connection::open(path).map_err(|error| io(&error))?;
    connection
        .busy_timeout(std::time::Duration::from_secs(2))
        .map_err(|error| io(&error))?;
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(|error| io(&error))?;
    migrate(&connection)?;
    Ok(connection)
}

/// Open an existing log for reading; `None` when nothing was recorded yet.
fn existing(path: &Path) -> Result<Option<Connection>, AppError> {
    if !path.exists() {
        return Ok(None);
    }
    open(path).map(Some)
}

fn migrate(connection: &Connection) -> Result<(), AppError> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| io(&error))?;
    match version {
        0 => {
            connection
                .execute_batch(&format!(
                    "BEGIN IMMEDIATE; {SCHEMA} PRAGMA user_version = {SCHEMA_VERSION}; COMMIT;"
                ))
                .map_err(|error| io(&error))?;
            Ok(())
        }
        SCHEMA_VERSION => Ok(()),
        newer => Err(AppError::configuration(format!(
            "the usage log is version {newer}, newer than this Mash ({SCHEMA_VERSION}); upgrade Mash to read it"
        ))),
    }
}

fn setting_enabled(connection: &Connection) -> Result<bool, AppError> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value FROM settings WHERE key = 'enabled'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| io(&error))?;
    Ok(value.as_deref() != Some("0"))
}

/// Store `rows` with references to what each returned, and add `dropped`
/// to the count of rows lost before they could be queued. A log turned off
/// stores neither. One short transaction per batch, so a reader is never
/// kept waiting.
pub(super) fn insert(
    connection: &mut Connection,
    rows: &[Call],
    dropped: u64,
) -> Result<(), AppError> {
    if !setting_enabled(connection)? {
        return Ok(());
    }
    let transaction = connection.transaction().map_err(|error| io(&error))?;
    for row in rows {
        let parsed = match &row.response {
            Some(super::Returned::Text(text)) => serde_json::from_str(text).ok(),
            Some(super::Returned::Json(value)) => Some(value.clone()),
            None => None,
        };
        let (references, total) = parsed.map_or_else(
            || (Vec::new(), None),
            |value: serde_json::Value| {
                let (references, total) = super::references::extract(&value);
                (references, Some(total))
            },
        );
        transaction
            .prepare_cached(
                "INSERT INTO call_rows (at_ms, surface, client, client_version, tool, arguments,
                 outcome, error_kind, error_message, duration_ms, result_count, response_bytes,
                 results_total, session, process_id, build, cwd)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            )
            .and_then(|mut statement| {
                statement.execute(params![
                    row.at_ms,
                    row.surface.as_str(),
                    row.client,
                    row.client_version,
                    row.tool,
                    row.arguments.to_string(),
                    row.outcome.as_str(),
                    row.error_kind,
                    row.error_message,
                    i64::try_from(row.duration_ms).unwrap_or(i64::MAX),
                    row.result_count.and_then(|count| i64::try_from(count).ok()),
                    row.response_bytes.and_then(|bytes| i64::try_from(bytes).ok()),
                    total.and_then(|total| i64::try_from(total).ok()),
                    row.session,
                    row.process_id,
                    row.build,
                    row.cwd,
                ])
            })
            .map_err(|error| io(&error))?;
        let call_id = transaction.last_insert_rowid();
        let mut statement = transaction
            .prepare_cached(
                "INSERT INTO result_rows (call_id, position, kind, record_kind, item_id, node)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(|error| io(&error))?;
        for (position, reference) in references.iter().enumerate() {
            statement
                .execute(params![
                    call_id,
                    i64::try_from(position).unwrap_or(i64::MAX),
                    reference.kind,
                    reference.record_kind,
                    reference.id,
                    reference.node,
                ])
                .map_err(|error| io(&error))?;
        }
    }
    if dropped > 0 {
        transaction
            .execute(
                "INSERT INTO counters (key, value) VALUES ('dropped', ?1)
                 ON CONFLICT (key) DO UPDATE SET value = value + excluded.value",
                [i64::try_from(dropped).unwrap_or(i64::MAX)],
            )
            .map_err(|error| io(&error))?;
    }
    transaction.commit().map_err(|error| io(&error))
}

/// Whether the log beside `database` records calls. On unless turned off.
pub fn enabled(database: &Path) -> Result<bool, AppError> {
    match existing(&super::path_for(database))? {
        Some(connection) => setting_enabled(&connection),
        None => Ok(true),
    }
}

/// Turn recording on or off for the log beside `database`.
pub fn set_enabled(database: &Path, on: bool) -> Result<(), AppError> {
    let connection = open(&super::path_for(database))?;
    connection
        .execute(
            "INSERT INTO settings (key, value) VALUES ('enabled', ?1)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            [if on { "1" } else { "0" }],
        )
        .map_err(|error| io(&error))?;
    Ok(())
}

/// Delete every recorded call and the drop count; returns how many calls were
/// deleted. The on/off setting is kept.
pub fn clear(database: &Path) -> Result<u64, AppError> {
    let Some(connection) = existing(&super::path_for(database))? else {
        return Ok(0);
    };
    connection
        .execute("DELETE FROM result_rows", [])
        .map_err(|error| io(&error))?;
    let deleted = connection
        .execute("DELETE FROM call_rows", [])
        .map_err(|error| io(&error))?;
    connection
        .execute("DELETE FROM counters", [])
        .map_err(|error| io(&error))?;
    connection
        .execute_batch("VACUUM")
        .map_err(|error| io(&error))?;
    Ok(u64::try_from(deleted).unwrap_or(0))
}

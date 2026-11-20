//! Durable transfer catalog and bounded piece spool.
//!
//! Bytes reach the filesystem before the checkpoint that acknowledges them is
//! committed to SQLite. A restart can therefore resume from the last reported
//! checkpoint, and a retry that reuses a sequence is accepted only when its
//! decoded digest matches the bytes already retained.
use super::wire;
use crate::AppError;
use rusqlite::{Connection, OptionalExtension, params};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

const BUSY_TIMEOUT: Duration = Duration::from_secs(30);
const SCHEMA_VERSION: i64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Receiving,
    QueuedForApply,
    Applying,
    ViewsPending,
    Published,
    Cancelled,
    Failed,
    Expired,
}

impl State {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Receiving => "receiving",
            Self::QueuedForApply => "queued_for_apply",
            Self::Applying => "applying",
            Self::ViewsPending => "views_pending",
            Self::Published => "published",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
            Self::Expired => "expired",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "receiving" => Some(Self::Receiving),
            "queued_for_apply" => Some(Self::QueuedForApply),
            "applying" => Some(Self::Applying),
            "views_pending" => Some(Self::ViewsPending),
            "published" => Some(Self::Published),
            "cancelled" => Some(Self::Cancelled),
            "failed" => Some(Self::Failed),
            "expired" => Some(Self::Expired),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::struct_field_names,
    reason = "the operation identity names the durable transfer row"
)]
pub struct Operation {
    pub operation_id: String,
    pub kind: Kind,
    pub manifest_digest: String,
    pub decoded_bytes: u64,
    pub record_count: u32,
    pub start_json: String,
    pub state: State,
    pub next_sequence: u64,
    pub decoded_offset: u64,
    pub revision: Option<i64>,
    pub receipt_digest: Option<String>,
}

pub struct Begin<'a> {
    pub operation_id: &'a str,
    pub group_id: &'a str,
    pub node_id: &'a str,
    pub kind: &'a Kind,
    pub manifest_digest: &'a str,
    pub decoded_bytes: u64,
    pub record_count: u32,
    pub start_json: &'a str,
}

pub enum BeginOutcome {
    New(Operation),
    Existing(Operation),
}

/// One durable progressive-backfill session. The manifest is stored in the
/// member table; `covered` counts members with committed newer evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::struct_field_names,
    reason = "the backfill identity names the durable session row"
)]
pub struct Backfill {
    pub backfill_id: String,
    pub database_instance: String,
    pub baseline_revision: i64,
    pub views_fingerprint: String,
    pub manifest_digest: String,
    pub total_chats: u64,
    pub covered: u64,
    pub state: String,
}

pub struct BackfillBegin<'a> {
    pub backfill_id: &'a str,
    pub group_id: &'a str,
    pub node_id: &'a str,
    pub database_instance: &'a str,
    pub baseline_revision: i64,
    pub views_fingerprint: &'a str,
    pub manifest_digest: &'a str,
    pub total_chats: u64,
}

/// A durable operation left behind by a process that stopped mid-apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recoverable {
    pub operation_id: String,
    pub group_id: String,
    pub node_id: String,
    pub kind: Kind,
}

pub struct StoredChunk {
    pub sequence: u64,
    pub decoded_offset: u64,
    pub decoded_length: u64,
    pub digest: String,
    pub encoded: Vec<u8>,
    pub compression: i32,
}

pub struct Catalog {
    root: PathBuf,
    connection: Connection,
    /// Total encoded bytes all retained pieces may occupy.
    limit_bytes: u64,
}

/// Spool ceiling for tests and tools that do not configure one.
#[cfg(any(test, feature = "test-support"))]
pub const DEFAULT_SPOOL_LIMIT: u64 = 2 * 1024 * 1024 * 1024;

fn database(error: rusqlite::Error) -> AppError {
    AppError::database(error)
}

fn conflict(message: &str) -> AppError {
    crate::shared::validation::conflict(message)
}

fn now() -> i64 {
    i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis()),
    )
    .unwrap_or(i64::MAX)
}

impl Catalog {
    #[cfg(any(test, feature = "test-support"))]
    pub fn open(root: &Path) -> Result<Self, AppError> {
        Self::open_with_limit(root, DEFAULT_SPOOL_LIMIT)
    }

    /// Open the catalog with a hard ceiling on retained spool bytes.
    pub fn open_with_limit(root: &Path, limit_bytes: u64) -> Result<Self, AppError> {
        std::fs::create_dir_all(root.join("pieces")).map_err(AppError::io)?;
        let path = root.join("catalog.sqlite3");
        let connection = Connection::open(&path).map_err(database)?;
        connection.busy_timeout(BUSY_TIMEOUT).map_err(database)?;
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(database)?;
        let populated: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table')",
                [],
                |row| row.get(0),
            )
            .map_err(database)?;
        if (populated || version != 0) && version != SCHEMA_VERSION {
            return Err(AppError::schema(format!(
                "unsupported sync catalog schema {version}; expected {SCHEMA_VERSION}"
            )));
        }
        connection
            .pragma_update(None, "journal_mode", "wal")
            .map_err(database)?;
        connection
            .pragma_update(None, "synchronous", "full")
            .map_err(database)?;
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS operations (
                    operation_id TEXT PRIMARY KEY,
                    group_id TEXT NOT NULL,
                    node_id TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    manifest_digest TEXT NOT NULL,
                    decoded_bytes INTEGER NOT NULL,
                    record_count INTEGER NOT NULL,
                    start_json TEXT NOT NULL,
                    state TEXT NOT NULL,
                    next_sequence INTEGER NOT NULL,
                    decoded_offset INTEGER NOT NULL,
                    revision INTEGER,
                    receipt_digest TEXT,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS pieces (
                    operation_id TEXT NOT NULL,
                    sequence INTEGER NOT NULL,
                    decoded_offset INTEGER NOT NULL,
                    decoded_length INTEGER NOT NULL,
                    digest TEXT NOT NULL,
                    encoded_length INTEGER NOT NULL,
                    compression INTEGER NOT NULL DEFAULT 1,
                    PRIMARY KEY (operation_id, sequence)
                );
                CREATE INDEX IF NOT EXISTS operations_owner
                    ON operations(group_id, node_id, manifest_digest);
                CREATE TABLE IF NOT EXISTS backfills (
                    backfill_id TEXT PRIMARY KEY,
                    group_id TEXT NOT NULL,
                    node_id TEXT NOT NULL,
                    database_instance TEXT NOT NULL,
                    baseline_revision INTEGER NOT NULL,
                    views_fingerprint TEXT NOT NULL,
                    manifest_digest TEXT NOT NULL,
                    total_chats INTEGER NOT NULL,
                    state TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS backfill_members (
                    backfill_id TEXT NOT NULL,
                    chat_id TEXT NOT NULL,
                    digest TEXT NOT NULL,
                    covered INTEGER NOT NULL DEFAULT 0,
                    live_revision INTEGER NOT NULL DEFAULT 0,
                    PRIMARY KEY (backfill_id, chat_id)
                );",
            )
            .map_err(database)?;
        // Only a new catalog needs stamping. Writing the version on every open
        // would commit under a transfer another connection has in progress.
        if version != SCHEMA_VERSION {
            connection
                .pragma_update(None, "user_version", SCHEMA_VERSION)
                .map_err(database)?;
        }
        Ok(Self {
            root: root.to_path_buf(),
            connection,
            limit_bytes,
        })
    }

    pub fn begin(&self, begin: &Begin<'_>) -> Result<BeginOutcome, AppError> {
        if begin.operation_id.is_empty() || begin.operation_id.len() > 4096 {
            return Err(AppError::usage("invalid sync operation identity"));
        }
        let existing = self.operation(begin.operation_id)?.is_some();
        if existing {
            let operation = self.select(begin.operation_id, begin.group_id, begin.node_id)?;
            if operation.manifest_digest != begin.manifest_digest
                || operation.decoded_bytes != begin.decoded_bytes
                || operation.kind != *begin.kind
            {
                return Err(conflict(
                    "sync operation identity was already used for different bytes",
                ));
            }
            // Same records, different start (revision, certificate): a new
            // publication, never a replay of the earlier receipt.
            if operation.start_json != begin.start_json {
                return Err(conflict(
                    "sync operation identity was already used for different publication metadata",
                ));
            }
            return Ok(BeginOutcome::Existing(operation));
        }
        let timestamp = now();
        self.connection
            .execute(
                "INSERT INTO operations (
                    operation_id, group_id, node_id, kind, manifest_digest,
                    decoded_bytes, record_count, start_json, state,
                    next_sequence, decoded_offset, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'receiving', 0, 0, ?9, ?9)",
                params![
                    begin.operation_id,
                    begin.group_id,
                    begin.node_id,
                    begin.kind,
                    begin.manifest_digest,
                    i64::try_from(begin.decoded_bytes)
                        .map_err(|_| AppError::usage("sync operation is too large"))?,
                    i64::from(begin.record_count),
                    begin.start_json,
                    timestamp
                ],
            )
            .map_err(database)?;
        Ok(BeginOutcome::New(self.select(
            begin.operation_id,
            begin.group_id,
            begin.node_id,
        )?))
    }

    /// Persist one validated piece, then advance the durable checkpoint.
    pub fn append(
        &self,
        operation_id: &str,
        group_id: &str,
        node_id: &str,
        chunk: &StoredChunk,
    ) -> Result<Operation, AppError> {
        let operation = self.select(operation_id, group_id, node_id)?;
        match operation.state {
            State::Receiving => {}
            State::Published => return Err(conflict("sync operation is already published")),
            _ => return Err(conflict("sync operation is not receiving data")),
        }
        if chunk.sequence < operation.next_sequence {
            let existing = self
                .piece(operation_id, chunk.sequence)?
                .ok_or_else(|| AppError::internal("retained transfer checkpoint has no piece"))?;
            if existing.digest != chunk.digest || existing.decoded_offset != chunk.decoded_offset {
                return Err(conflict("conflicting retry for a retained sync piece"));
            }
            return Ok(operation);
        }
        if chunk.sequence != operation.next_sequence {
            return Err(conflict("sync piece sequence is not contiguous"));
        }
        if chunk.decoded_offset != operation.decoded_offset {
            return Err(conflict("sync piece offset is not contiguous"));
        }
        let end = operation
            .decoded_offset
            .checked_add(chunk.decoded_length)
            .ok_or_else(|| AppError::usage("sync piece length overflow"))?;
        if end > operation.decoded_bytes {
            return Err(AppError::usage(
                "sync pieces exceed the declared decoded size",
            ));
        }
        self.write_piece(operation_id, chunk)?;
        let transaction = self.write()?;
        let retained: i64 = transaction
            .query_row(
                "SELECT COALESCE(SUM(encoded_length), 0) FROM pieces",
                [],
                |row| row.get(0),
            )
            .map_err(database)?;
        let incoming = i64::try_from(chunk.encoded.len())
            .map_err(|_| AppError::usage("sync piece is too large"))?;
        if retained.saturating_add(incoming) > i64::try_from(self.limit_bytes).unwrap_or(i64::MAX) {
            transaction.rollback().map_err(database)?;
            let _ = std::fs::remove_file(self.piece_path(operation_id, chunk.sequence));
            return Err(AppError::shared(
                "sync spool quota exceeded; retry after other transfers finish or expire",
            ));
        }
        transaction
            .execute(
                "INSERT INTO pieces (
                    operation_id, sequence, decoded_offset, decoded_length,
                    digest, encoded_length, compression
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    operation_id,
                    i64::try_from(chunk.sequence)
                        .map_err(|_| AppError::usage("sync sequence is too large"))?,
                    i64::try_from(chunk.decoded_offset)
                        .map_err(|_| AppError::usage("sync offset is too large"))?,
                    i64::try_from(chunk.decoded_length)
                        .map_err(|_| AppError::usage("sync piece is too large"))?,
                    chunk.digest,
                    i64::try_from(chunk.encoded.len())
                        .map_err(|_| AppError::usage("sync piece is too large"))?,
                    chunk.compression,
                ],
            )
            .map_err(database)?;
        let changed = transaction
            .execute(
                "UPDATE operations SET next_sequence = ?2, decoded_offset = ?3, updated_at = ?4
                 WHERE operation_id = ?1 AND state = 'receiving' AND next_sequence = ?5",
                params![
                    operation_id,
                    i64::try_from(chunk.sequence + 1)
                        .map_err(|_| AppError::usage("sync sequence is too large"))?,
                    i64::try_from(end).map_err(|_| AppError::usage("sync offset is too large"))?,
                    now(),
                    i64::try_from(chunk.sequence)
                        .map_err(|_| AppError::usage("sync sequence is too large"))?,
                ],
            )
            .map_err(database)?;
        if changed != 1 {
            // A concurrent expiry or takeover closed this transfer between the
            // state check above and this transaction; fail closed.
            transaction.rollback().map_err(database)?;
            return Err(conflict(
                "sync operation is no longer receiving this transfer",
            ));
        }
        transaction.commit().map_err(database)?;
        self.select(operation_id, group_id, node_id)
    }

    /// Verify the complete receipt and queue it for application.
    pub fn finish(
        &self,
        operation_id: &str,
        group_id: &str,
        node_id: &str,
        manifest_digest: &str,
    ) -> Result<(Operation, Vec<wire::Piece>), AppError> {
        let operation = self.select(operation_id, group_id, node_id)?;
        if operation.manifest_digest != manifest_digest {
            return Err(conflict(
                "sync manifest does not match the received operation",
            ));
        }
        if operation.decoded_offset != operation.decoded_bytes {
            return Err(conflict("sync operation is incomplete"));
        }
        let pieces = self.pieces(operation_id)?;
        if pieces.len() as u64 != operation.next_sequence {
            return Err(AppError::internal("durable sync checkpoint lost pieces"));
        }
        let entries = pieces
            .iter()
            .map(|piece| wire::ManifestPiece {
                sequence: piece.sequence,
                decoded_offset: piece.decoded_offset,
                decoded_length: piece.decoded_length,
                digest: &piece.digest,
            })
            .collect::<Vec<_>>();
        if wire::manifest_digest(operation.decoded_bytes, operation.record_count, &entries)
            != operation.manifest_digest
        {
            return Err(conflict("sync manifest does not match the received pieces"));
        }
        if matches!(operation.state, State::Receiving) {
            self.set_state(
                operation_id,
                State::QueuedForApply,
                operation.revision,
                operation.receipt_digest.as_deref(),
            )?;
        }
        Ok((self.select(operation_id, group_id, node_id)?, pieces))
    }

    /// Operations that reached a durable apply state before the process
    /// stopped. Receiving transfers are not listed: they wait for their client.
    pub fn recoverable(&self) -> Result<Vec<Recoverable>, AppError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT operation_id, group_id, node_id, kind FROM operations
                 WHERE state IN ('queued_for_apply', 'applying', 'views_pending')
                 ORDER BY created_at, operation_id",
            )
            .map_err(database)?;
        let rows = statement
            .query_map([], |row| {
                Ok(Recoverable {
                    operation_id: row.get(0)?,
                    group_id: row.get(1)?,
                    node_id: row.get(2)?,
                    kind: row.get(3)?,
                })
            })
            .map_err(database)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(database)?;
        Ok(rows)
    }

    /// Retained pieces of a recoverable operation, with ownership checks.
    pub fn recover_pieces(
        &self,
        operation_id: &str,
        group_id: &str,
        node_id: &str,
    ) -> Result<(Operation, Vec<wire::Piece>), AppError> {
        let operation = self.select(operation_id, group_id, node_id)?;
        let pieces = self.pieces(operation_id)?;
        Ok((operation, pieces))
    }

    pub fn select(
        &self,
        operation_id: &str,
        group_id: &str,
        node_id: &str,
    ) -> Result<Operation, AppError> {
        let operation = self
            .operation(operation_id)?
            .ok_or_else(|| conflict("unknown sync operation"))?;
        if operation.0 != group_id || operation.1 != node_id {
            return Err(conflict("sync operation belongs to another database"));
        }
        Ok(operation.2)
    }

    /// Operation lookup used only to distinguish an unknown operation from an
    /// operation owned by another authenticated node.
    fn operation(
        &self,
        operation_id: &str,
    ) -> Result<Option<(String, String, Operation)>, AppError> {
        self.connection
            .query_row(
                "SELECT group_id, node_id, kind, manifest_digest, decoded_bytes,
                        record_count, start_json, state, next_sequence,
                        decoded_offset, revision, receipt_digest
                 FROM operations WHERE operation_id = ?1",
                params![operation_id],
                |row| {
                    let state: String = row.get(7)?;
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        Operation {
                            operation_id: operation_id.to_owned(),
                            kind: row.get(2)?,
                            manifest_digest: row.get(3)?,
                            decoded_bytes: row.get::<_, i64>(4)?.try_into().unwrap_or_default(),
                            record_count: row.get::<_, i64>(5)?.try_into().unwrap_or_default(),
                            start_json: row.get(6)?,
                            state: State::parse(&state).ok_or(rusqlite::Error::InvalidQuery)?,
                            next_sequence: row.get::<_, i64>(8)?.try_into().unwrap_or_default(),
                            decoded_offset: row.get::<_, i64>(9)?.try_into().unwrap_or_default(),
                            revision: row.get(10)?,
                            receipt_digest: row.get(11)?,
                        },
                    ))
                },
            )
            .optional()
            .map_err(database)
    }

    pub fn set_state(
        &self,
        operation_id: &str,
        state: State,
        revision: Option<i64>,
        receipt_digest: Option<&str>,
    ) -> Result<(), AppError> {
        self.connection
            .execute(
                "UPDATE operations SET state = ?2, revision = ?3, receipt_digest = ?4,
                        updated_at = ?5 WHERE operation_id = ?1",
                params![
                    operation_id,
                    state.as_str(),
                    revision,
                    receipt_digest,
                    now()
                ],
            )
            .map_err(database)?;
        Ok(())
    }

    pub fn published(
        &self,
        operation_id: &str,
        revision: i64,
        receipt_digest: &str,
    ) -> Result<(), AppError> {
        self.set_state(
            operation_id,
            State::Published,
            Some(revision),
            Some(receipt_digest),
        )?;
        self.remove_pieces(operation_id)
    }

    /// Mark a non-publication operation complete without claiming a silver
    /// revision. Used for the backfill manifest transfer.
    pub fn completed(&self, operation_id: &str, receipt_digest: &str) -> Result<(), AppError> {
        self.set_state(operation_id, State::Published, None, Some(receipt_digest))?;
        self.remove_pieces(operation_id)
    }

    pub fn cancel(&self, operation_id: &str) -> Result<(), AppError> {
        self.set_state(operation_id, State::Cancelled, None, None)?;
        self.remove_pieces(operation_id)
    }

    /// A write transaction that takes the write lock at once. A deferred one
    /// that reads first cannot wait for another connection's commit: SQLite
    /// refuses it outright, and the transfer it belonged to is abandoned.
    fn write(&self) -> Result<rusqlite::Transaction<'_>, AppError> {
        rusqlite::Transaction::new_unchecked(
            &self.connection,
            rusqlite::TransactionBehavior::Immediate,
        )
        .map_err(database)
    }

    /// Expire abandoned receiving transfers whose last progress is older than
    /// `cutoff` (milliseconds since the epoch). Completed work is never expired
    /// here: only `receiving` rows release their spool.
    pub fn expire_incomplete(&self, cutoff: i64) -> Result<usize, AppError> {
        // One immediate transaction: an in-flight append on another connection
        // either commits first (and changes `updated_at`) or waits. That keeps a
        // checkpoint acknowledged by `append` from being deleted here.
        let transaction = rusqlite::Transaction::new_unchecked(
            &self.connection,
            rusqlite::TransactionBehavior::Immediate,
        )
        .map_err(database)?;
        let ids = {
            let mut statement = transaction
                .prepare(
                    "SELECT operation_id FROM operations
                     WHERE state = 'receiving' AND updated_at < ?1",
                )
                .map_err(database)?;
            statement
                .query_map(params![cutoff], |row| row.get::<_, String>(0))
                .map_err(database)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(database)?
        };
        let mut expired = Vec::new();
        for id in ids {
            let changed = transaction
                .execute(
                    "UPDATE operations SET state = 'expired', updated_at = ?2
                     WHERE operation_id = ?1 AND state = 'receiving' AND updated_at < ?3",
                    params![id, now(), cutoff],
                )
                .map_err(database)?;
            if changed == 1 {
                expired.push(id);
            }
        }
        for id in &expired {
            transaction
                .execute("DELETE FROM pieces WHERE operation_id = ?1", params![id])
                .map_err(database)?;
        }
        transaction.commit().map_err(database)?;
        // Files go after the durable state change; an interrupted sweep can
        // leave unreferenced piece files, never a checkpoint without bytes.
        for id in &expired {
            match std::fs::remove_dir_all(self.piece_directory(id)) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(AppError::io(error)),
            }
        }
        Ok(expired.len())
    }

    /// Forget every transfer, receipt and backfill of one node. A receipt kept
    /// after its history is gone could acknowledge an upload that no longer
    /// exists, so retiring a node removes them with it.
    pub fn retire(&self, group_id: &str, node_id: &str) -> Result<usize, AppError> {
        let transaction = rusqlite::Transaction::new_unchecked(
            &self.connection,
            rusqlite::TransactionBehavior::Immediate,
        )
        .map_err(database)?;
        let ids = {
            let mut statement = transaction
                .prepare("SELECT operation_id FROM operations WHERE group_id = ?1 AND node_id = ?2")
                .map_err(database)?;
            statement
                .query_map(params![group_id, node_id], |row| row.get::<_, String>(0))
                .map_err(database)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(database)?
        };
        for id in &ids {
            transaction
                .execute("DELETE FROM pieces WHERE operation_id = ?1", params![id])
                .map_err(database)?;
        }
        transaction
            .execute(
                "DELETE FROM backfill_members WHERE backfill_id IN
                    (SELECT backfill_id FROM backfills WHERE group_id = ?1 AND node_id = ?2)",
                params![group_id, node_id],
            )
            .map_err(database)?;
        for table in ["backfills", "operations"] {
            transaction
                .execute(
                    &format!("DELETE FROM {table} WHERE group_id = ?1 AND node_id = ?2"),
                    params![group_id, node_id],
                )
                .map_err(database)?;
        }
        transaction.commit().map_err(database)?;
        // Files go after the durable change, as when a transfer expires.
        for id in &ids {
            match std::fs::remove_dir_all(self.piece_directory(id)) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(AppError::io(error)),
            }
        }
        Ok(ids.len())
    }

    /// Bound retained spool and catalog rows. Published operations keep only
    /// their receipt so a reconnect can still report the outcome.
    pub fn prune(&self, group_id: &str, node_id: &str, keep: usize) -> Result<(), AppError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT operation_id FROM operations
                 WHERE group_id = ?1 AND node_id = ?2
                   AND state IN ('published', 'cancelled', 'failed', 'expired')
                 ORDER BY updated_at DESC LIMIT -1 OFFSET ?3",
            )
            .map_err(database)?;
        let ids = statement
            .query_map(
                params![group_id, node_id, i64::try_from(keep).unwrap_or(i64::MAX)],
                |row| row.get::<_, String>(0),
            )
            .map_err(database)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(database)?;
        drop(statement);
        for id in ids {
            self.remove_pieces(&id)?;
            self.connection
                .execute(
                    "DELETE FROM operations WHERE operation_id = ?1",
                    params![id],
                )
                .map_err(database)?;
        }
        Ok(())
    }

    fn pieces(&self, operation_id: &str) -> Result<Vec<wire::Piece>, AppError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT sequence, decoded_offset, decoded_length, digest, compression
                 FROM pieces WHERE operation_id = ?1 ORDER BY sequence",
            )
            .map_err(database)?;
        let rows = statement
            .query_map(params![operation_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i32>(4)?,
                ))
            })
            .map_err(database)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(database)?;
        drop(statement);
        let directory = self.piece_directory(operation_id);
        rows.into_iter()
            .map(|(sequence, offset, length, digest, compression)| {
                let sequence = u64::try_from(sequence)
                    .map_err(|_| AppError::internal("invalid retained sequence"))?;
                let encoded = std::fs::read(directory.join(format!("{sequence}.zst")))
                    .map_err(|_| AppError::internal("read retained sync piece"))?;
                Ok(wire::Piece {
                    sequence,
                    decoded_offset: u64::try_from(offset)
                        .map_err(|_| AppError::internal("invalid retained offset"))?,
                    decoded_length: u64::try_from(length)
                        .map_err(|_| AppError::internal("invalid retained length"))?,
                    digest,
                    encoded,
                    compression,
                })
            })
            .collect()
    }

    fn piece(&self, operation_id: &str, sequence: u64) -> Result<Option<StoredChunk>, AppError> {
        let row = self
            .connection
            .query_row(
                "SELECT decoded_offset, decoded_length, digest, compression
                 FROM pieces WHERE operation_id = ?1 AND sequence = ?2",
                params![
                    operation_id,
                    i64::try_from(sequence).map_err(|_| AppError::usage("invalid sequence"))?
                ],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i32>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(database)?;
        let Some((offset, length, digest, compression)) = row else {
            return Ok(None);
        };
        Ok(Some(StoredChunk {
            sequence,
            decoded_offset: u64::try_from(offset)
                .map_err(|_| AppError::internal("invalid retained offset"))?,
            decoded_length: u64::try_from(length)
                .map_err(|_| AppError::internal("invalid retained length"))?,
            digest,
            encoded: Vec::new(),
            compression,
        }))
    }

    fn piece_path(&self, operation_id: &str, sequence: u64) -> PathBuf {
        self.piece_directory(operation_id)
            .join(format!("{sequence}.zst"))
    }

    fn piece_directory(&self, operation_id: &str) -> PathBuf {
        self.root
            .join("pieces")
            .join(blake3::hash(operation_id.as_bytes()).to_hex().as_str())
    }

    fn write_piece(&self, operation_id: &str, chunk: &StoredChunk) -> Result<(), AppError> {
        let directory = self.piece_directory(operation_id);
        std::fs::create_dir_all(&directory).map_err(AppError::io)?;
        let path = directory.join(format!("{}.zst", chunk.sequence));
        let existing = std::fs::read(&path).ok();
        if let Some(existing) = existing {
            if existing == chunk.encoded {
                return Ok(());
            }
            return Err(conflict("conflicting retry for a retained sync piece"));
        }
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(AppError::io)?;
        file.write_all(&chunk.encoded).map_err(AppError::io)?;
        file.sync_all().map_err(AppError::io)?;
        drop(file);
        if let Ok(directory) = std::fs::File::open(&directory) {
            let _ = directory.sync_all();
        }
        Ok(())
    }

    fn remove_pieces(&self, operation_id: &str) -> Result<(), AppError> {
        let directory = self.piece_directory(operation_id);
        match std::fs::remove_dir_all(&directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(AppError::io(error)),
        }
        self.connection
            .execute(
                "DELETE FROM pieces WHERE operation_id = ?1",
                params![operation_id],
            )
            .map_err(database)?;
        Ok(())
    }
}

mod backfill;
mod kind;
pub use kind::Kind;

#[cfg(test)]
mod tests;

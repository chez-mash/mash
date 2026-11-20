//! The catalog's progressive-backfill records: a baseline, its manifest and
//! membership, coverage, and which baselines live publications supersede.
use super::{
    AppError, Backfill, BackfillBegin, Catalog, OptionalExtension, conflict, database, now, params,
    wire,
};

impl Catalog {
    /// Create or re-open a progressive-backfill session. Identity and manifest
    /// are immutable once the session exists.
    pub fn begin_backfill(&self, input: &BackfillBegin<'_>) -> Result<Backfill, AppError> {
        if input.backfill_id.is_empty() || input.backfill_id.len() > 4096 {
            return Err(AppError::usage("invalid backfill identity"));
        }
        if input.baseline_revision < 0
            || input.database_instance.is_empty()
            || input.views_fingerprint.is_empty()
            || input.manifest_digest.len() != 64
            || !input.manifest_digest.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(AppError::usage("invalid backfill manifest"));
        }
        let timestamp = now();
        self.connection
            .execute(
                "INSERT OR IGNORE INTO backfills (
                    backfill_id, group_id, node_id, database_instance,
                    baseline_revision, views_fingerprint, manifest_digest,
                    total_chats, state, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'receiving_manifest', ?9, ?9)",
                params![
                    input.backfill_id,
                    input.group_id,
                    input.node_id,
                    input.database_instance,
                    input.baseline_revision,
                    input.views_fingerprint,
                    input.manifest_digest,
                    i64::try_from(input.total_chats)
                        .map_err(|_| AppError::usage("backfill is too large"))?,
                    timestamp
                ],
            )
            .map_err(database)?;
        let backfill = self.backfill(input.backfill_id, input.group_id, input.node_id)?;
        if backfill.database_instance != input.database_instance
            || backfill.baseline_revision != input.baseline_revision
            || backfill.views_fingerprint != input.views_fingerprint
            || backfill.manifest_digest != input.manifest_digest
            || backfill.total_chats != input.total_chats
        {
            return Err(conflict(
                "backfill identity was already used for a different baseline",
            ));
        }
        Ok(backfill)
    }

    pub fn backfill(
        &self,
        backfill_id: &str,
        group_id: &str,
        node_id: &str,
    ) -> Result<Backfill, AppError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT b.database_instance, b.baseline_revision, b.views_fingerprint,
                        b.manifest_digest, b.total_chats, b.state,
                        (SELECT COUNT(*) FROM backfill_members m
                         WHERE m.backfill_id = b.backfill_id AND m.covered = 1)
                 FROM backfills b
                 WHERE b.backfill_id = ?1 AND b.group_id = ?2 AND b.node_id = ?3",
            )
            .map_err(database)?;
        statement
            .query_row(params![backfill_id, group_id, node_id], |row| {
                Ok(Backfill {
                    backfill_id: backfill_id.to_owned(),
                    database_instance: row.get(0)?,
                    baseline_revision: row.get(1)?,
                    views_fingerprint: row.get(2)?,
                    manifest_digest: row.get(3)?,
                    total_chats: row.get::<_, i64>(4)?.try_into().unwrap_or_default(),
                    covered: row.get::<_, i64>(6)?.try_into().unwrap_or_default(),
                    state: row.get(5)?,
                })
            })
            .optional()
            .map_err(database)?
            .ok_or_else(|| conflict("unknown backfill operation"))
    }

    /// Install the verified manifest once its transfer finishes. Repeating the
    /// same manifest is idempotent; different bytes fail closed.
    pub fn store_manifest(
        &self,
        backfill_id: &str,
        group_id: &str,
        node_id: &str,
        members: &[wire::BackfillMember],
    ) -> Result<Backfill, AppError> {
        let backfill = self.backfill(backfill_id, group_id, node_id)?;
        if backfill.total_chats != members.len() as u64 {
            return Err(conflict(
                "backfill manifest member count does not match its declaration",
            ));
        }
        if backfill.state == "receiving_manifest" {
            let transaction = self.write()?;
            for member in members {
                transaction
                    .execute(
                        "INSERT OR IGNORE INTO backfill_members (
                            backfill_id, chat_id, digest, covered
                         ) VALUES (?1, ?2, ?3, 0)",
                        params![backfill_id, member.chat_id, member.digest],
                    )
                    .map_err(database)?;
            }
            let retained: i64 = transaction
                .query_row(
                    "SELECT COUNT(*) FROM backfill_members WHERE backfill_id = ?1",
                    params![backfill_id],
                    |row| row.get(0),
                )
                .map_err(database)?;
            if retained != i64::try_from(members.len()).unwrap_or(i64::MAX) {
                return Err(conflict(
                    "backfill manifest was already installed with different members",
                ));
            }
            transaction
                .execute(
                    "UPDATE backfills SET state = 'ready', updated_at = ?2
                     WHERE backfill_id = ?1",
                    params![backfill_id, now()],
                )
                .map_err(database)?;
            transaction.commit().map_err(database)?;
        }
        self.backfill(backfill_id, group_id, node_id)
    }

    /// Mark baseline members covered by a committed batch. Members outside the
    /// manifest are live additions and intentionally ignored.
    pub fn cover(
        &self,
        backfill_id: &str,
        group_id: &str,
        node_id: &str,
        members: &[(String, String)],
    ) -> Result<u64, AppError> {
        let backfill = self.backfill(backfill_id, group_id, node_id)?;
        if backfill.state != "ready" && backfill.state != "complete" {
            return Err(conflict("backfill manifest is not ready"));
        }
        let transaction = self.write()?;
        for (chat_id, digest) in members {
            transaction
                .execute(
                    "UPDATE backfill_members SET covered = 1
                     WHERE backfill_id = ?1 AND chat_id = ?2 AND digest = ?3",
                    params![backfill_id, chat_id, digest],
                )
                .map_err(database)?;
        }
        transaction.commit().map_err(database)?;
        Ok(self.backfill(backfill_id, group_id, node_id)?.covered)
    }

    /// Verify complete manifest coverage. Idempotent after completion.
    pub fn finish_backfill(
        &self,
        backfill_id: &str,
        group_id: &str,
        node_id: &str,
        baseline_revision: i64,
        manifest_digest: &str,
    ) -> Result<Backfill, AppError> {
        let backfill = self.backfill(backfill_id, group_id, node_id)?;
        if backfill.state == "complete" {
            return Ok(backfill);
        }
        if backfill.state != "ready" {
            return Err(conflict("backfill manifest is not complete"));
        }
        if backfill.baseline_revision != baseline_revision
            || backfill.manifest_digest != manifest_digest
        {
            return Err(conflict(
                "backfill finish does not match the installed baseline",
            ));
        }
        if backfill.covered != backfill.total_chats {
            return Err(conflict("backfill baseline coverage is incomplete"));
        }
        self.connection
            .execute(
                "UPDATE backfills SET state = 'complete', updated_at = ?2
                 WHERE backfill_id = ?1",
                params![backfill_id, now()],
            )
            .map_err(database)?;
        self.backfill(backfill_id, group_id, node_id)
    }

    /// Backfill sessions that still accept baseline batches for this node.
    pub fn active_backfills(&self, group_id: &str, node_id: &str) -> Result<Vec<String>, AppError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT backfill_id FROM backfills
                 WHERE group_id = ?1 AND node_id = ?2
                   AND state IN ('ready', 'complete')
                 ORDER BY created_at, backfill_id",
            )
            .map_err(database)?;
        let rows = statement
            .query_map(params![group_id, node_id], |row| row.get::<_, String>(0))
            .map_err(database)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(database)?;
        Ok(rows)
    }

    /// Record that a newer live publication carries these chats. A
    /// live record at a revision above the frozen baseline satisfies its
    /// baseline member, and a later baseline batch must not overwrite it.
    pub fn record_live(
        &self,
        backfill_id: &str,
        group_id: &str,
        node_id: &str,
        chats: &[String],
        revision: i64,
    ) -> Result<(), AppError> {
        let backfill = self.backfill(backfill_id, group_id, node_id)?;
        if revision <= backfill.baseline_revision {
            return Ok(());
        }
        let transaction = self.write()?;
        for chat in chats {
            transaction
                .execute(
                    "UPDATE backfill_members
                     SET live_revision = MAX(live_revision, ?2),
                         covered = CASE WHEN ?2 > ?3 THEN 1 ELSE covered END
                     WHERE backfill_id = ?1 AND chat_id = ?4",
                    params![backfill_id, revision, backfill.baseline_revision, chat],
                )
                .map_err(database)?;
        }
        transaction.commit().map_err(database)?;
        Ok(())
    }

    /// Baseline chats already superseded by a live publication at or
    /// above `revision`. Their baseline records must not be applied.
    pub fn superseded(
        &self,
        backfill_id: &str,
        group_id: &str,
        node_id: &str,
        revision: i64,
    ) -> Result<Vec<String>, AppError> {
        self.backfill(backfill_id, group_id, node_id)?;
        let mut statement = self
            .connection
            .prepare(
                "SELECT chat_id FROM backfill_members
                 WHERE backfill_id = ?1 AND live_revision > ?2
                 ORDER BY chat_id",
            )
            .map_err(database)?;
        let rows = statement
            .query_map(params![backfill_id, revision], |row| {
                row.get::<_, String>(0)
            })
            .map_err(database)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(database)?;
        Ok(rows)
    }

    /// Most recently created baseline session for a node.
    pub fn latest_backfill(
        &self,
        group_id: &str,
        node_id: &str,
    ) -> Result<Option<Backfill>, AppError> {
        let id = self
            .connection
            .query_row(
                "SELECT backfill_id FROM backfills
                 WHERE group_id = ?1 AND node_id = ?2
                 ORDER BY created_at DESC, backfill_id DESC LIMIT 1",
                params![group_id, node_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(database)?;
        match id {
            Some(id) => Ok(Some(self.backfill(&id, group_id, node_id)?)),
            None => Ok(None),
        }
    }
}

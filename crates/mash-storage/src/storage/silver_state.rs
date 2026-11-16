//! Durable silver evidence; independent of embeddings and other views writes.
use super::{Storage, generated};
use crate::AppError;
use crate::provider::Provider;
use crate::silver::certificate::{Certificate, Completeness, INTERPRETATION_VERSION};
use crate::silver::{SourceIdentity, SourceStamp};
use rusqlite::params;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Clone, Copy)]
#[repr(i64)]
enum ScanState {
    Pending = 0,
    Complete = 1,
    Incomplete = 2,
}

/// How many of one provider's sources are in a state, for diagnosis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCount {
    pub provider: String,
    pub sources: u64,
    /// Of those, the ones last active before the history range began.
    pub outside_range: u64,
}

impl Storage {
    pub fn silver_certificate(&self) -> Result<Certificate, AppError> {
        self.silver_certificate_for(INTERPRETATION_VERSION)
    }

    pub(super) fn silver_certificate_for(
        &self,
        interpretation: u32,
    ) -> Result<Certificate, AppError> {
        let row = generated::SilverCertificate::builder()
            .current_version(i64::from(interpretation))
            .build()
            .query_one(&self.connection)
            .map_err(AppError::database)?;
        let completeness = if row.pending != 0 {
            Completeness::Pending
        } else if row.unverified != 0 {
            Completeness::Unverified
        } else if row.incomplete != 0 || row.rejected_sources != 0 {
            Completeness::Incomplete
        } else {
            Completeness::Complete
        };
        Ok(Certificate {
            revision: u64::try_from(row.revision)
                .map_err(|_| AppError::database_data("invalid silver revision"))?,
            interpretation_version: u32::try_from(row.interpretation_version)
                .map_err(|_| AppError::database_data("invalid silver interpretation version"))?,
            completeness,
            rejected_sources: u64::try_from(row.rejected_sources)
                .map_err(|_| AppError::database_data("invalid silver rejection count"))?,
        })
    }

    /// Per provider, the sources whose checkpoint an earlier reader wrote,
    /// and how many of them were last active before `cutoff_modified_ns`:
    /// those an ordinary pass will not admit. For diagnosis only.
    pub fn sources_awaiting_reading(
        &self,
        cutoff_modified_ns: Option<i64>,
    ) -> Result<Vec<SourceCount>, AppError> {
        self.source_counts(
            "SELECT provider, count(*), coalesce(sum(?2 IS NOT NULL AND modified_ns < ?2), 0)
               FROM source_checkpoints WHERE interpretation_version <> ?1
              GROUP BY provider ORDER BY provider",
            params![INTERPRETATION_VERSION, cutoff_modified_ns],
        )
    }

    /// Per provider, the sources the last pass could not read.
    pub fn rejected_sources(&self) -> Result<Vec<SourceCount>, AppError> {
        self.source_counts(
            "SELECT provider, count(*), 0 FROM silver_rejections
              GROUP BY provider ORDER BY provider",
            [],
        )
    }

    fn source_counts(
        &self,
        sql: &str,
        parameters: impl rusqlite::Params,
    ) -> Result<Vec<SourceCount>, AppError> {
        let count = |value: i64| u64::try_from(value).unwrap_or_default();
        self.connection
            .prepare(sql)
            .map_err(AppError::database)?
            .query_map(parameters, |row| {
                Ok(SourceCount {
                    provider: row.get(0)?,
                    sources: count(row.get(1)?),
                    outside_range: count(row.get(2)?),
                })
            })
            .map_err(AppError::database)?
            .collect::<Result<_, _>>()
            .map_err(AppError::database)
    }

    pub fn begin_silver_scan(&mut self, provider: Provider) -> Result<(), AppError> {
        self.require_writer()?;
        self.connection.execute(
            "INSERT INTO silver_scans(provider,interpretation_version,state) VALUES(?1,?2,?3)
             ON CONFLICT(provider) DO UPDATE SET interpretation_version=excluded.interpretation_version,state=excluded.state",
            params![provider, INTERPRETATION_VERSION, ScanState::Pending as i64],
        ).map_err(AppError::database)?;
        // Pending silver certification makes all previous gold builds stale.
        Ok(())
    }

    pub fn finish_silver_scan(
        &mut self,
        provider: Provider,
        complete: bool,
    ) -> Result<(), AppError> {
        self.require_writer()?;
        let state = if complete {
            ScanState::Complete
        } else {
            ScanState::Incomplete
        };
        self.connection
            .execute(
                "UPDATE silver_scans SET state=?2 WHERE provider=?1",
                params![provider, state as i64],
            )
            .map_err(AppError::database)?;
        Ok(())
    }

    pub fn finish_silver_ingestion(&mut self) -> Result<(), AppError> {
        self.require_writer()?;
        self.connection
            .execute(
                "UPDATE silver_state SET interpretation_version=?1 WHERE singleton=1",
                [INTERPRETATION_VERSION],
            )
            .map_err(AppError::database)?;
        Ok(())
    }

    pub fn reject_silver_source(
        &mut self,
        provider: Provider,
        source: &SourceIdentity,
        stamp: &SourceStamp,
    ) -> Result<(), AppError> {
        self.require_writer()?;
        self.connection.execute(
            "INSERT INTO silver_rejections(provider,source_key,source_path,modified_ns) VALUES(?1,?2,?3,?4)
             ON CONFLICT(provider,source_key) DO UPDATE SET source_path=excluded.source_path,modified_ns=excluded.modified_ns",
            params![provider, source.key(), source.path.to_string_lossy(), stamp.activity_ns.unwrap_or(stamp.modified_ns)],
        ).map_err(AppError::database)?;
        Ok(())
    }

    pub(super) fn clear_silver_rejection(
        &mut self,
        provider: Provider,
        key: &str,
    ) -> Result<(), AppError> {
        self.connection
            .execute(
                "DELETE FROM silver_rejections WHERE provider=?1 AND source_key=?2",
                params![provider, key],
            )
            .map_err(AppError::database)?;
        Ok(())
    }

    pub(super) fn purge_missing_silver_rejections(
        &mut self,
        provider: Provider,
        observed: &BTreeSet<String>,
        roots: &[PathBuf],
        cutoff: Option<i64>,
    ) -> Result<(), AppError> {
        let records = generated::SilverRejections::builder()
            .provider(provider.as_str())
            .build()
            .query_many(&self.connection)
            .map_err(AppError::database)?;
        for record in records {
            if !observed.contains(&record.source_key)
                && super::indexing::source_is_in_reconciliation_scope(
                    &record.source_path,
                    Some(record.modified_ns),
                    roots,
                    cutoff,
                )
            {
                self.clear_silver_rejection(provider, &record.source_key)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

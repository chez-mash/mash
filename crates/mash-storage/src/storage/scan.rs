//! Word search without Qdrant: a bounded scan of the saved messages.
//!
//! Search normally reads Qdrant, which holds the ranked word and meaning
//! indexes. When it is not running, history saved in SQLite is still here, so
//! a plain word search can scan it instead: each word must appear in the
//! message as a whole word, case-insensitively, and the newest matches come
//! first. The scan
//! reads what search would index (a message's search projection, or its
//! content when it has none) and stops at [`MATCH_LIMIT`] matches or after
//! [`TIME_BUDGET`], reporting when it stopped early.
use super::Storage;
use crate::AppError;
use crate::search_results::ChatMetadata;
use crate::storage::SearchHit;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// The most matching messages one scan keeps.
pub const MATCH_LIMIT: usize = 2_000;
/// How long one scan may read before it returns what it found.
pub const TIME_BUDGET: Duration = Duration::from_secs(10);

/// What a scan found, and whether it read every saved message.
pub struct Scan {
    pub hits: Vec<SearchHit>,
    pub complete: bool,
}

impl Storage {
    /// Messages containing every one of `needles` as a whole word, newest
    /// first, within `scope`. The needles are words as
    /// [`crate::views::matching::tokens`] splits them: lowercase, letters and
    /// digits only.
    pub fn scan_messages(
        &mut self,
        needles: &[String],
        scope: &crate::search_filter::Scope,
    ) -> Result<Scan, AppError> {
        let filter = scope.filter.sql(Some(&scope.node));
        // Each message is decoded once, here, and matched in Rust: a check in
        // SQL would decode it again for every word.
        let sql = format!(
            "SELECT m.id,COALESCE(m.search_projection,m.content) AS text
             FROM messages m JOIN chats c ON c.id=m.chat_id WHERE ({filter})"
        );
        let started = Instant::now();
        self.connection
            .progress_handler(10_000, Some(move || started.elapsed() > TIME_BUDGET))
            .map_err(AppError::database)?;
        let found = self.matching_ids(&sql, needles);
        self.connection
            .progress_handler(0, None::<fn() -> bool>)
            .map_err(AppError::database)?;
        let (ids, complete) = found?;
        let mut statement = self.connection.prepare(HIT).map_err(AppError::database)?;
        let mut hits = ids
            .iter()
            .map(|id| statement.query_row([id], hit))
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(AppError::database)?;
        hits.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(Scan { hits, complete })
    }

    fn matching_ids(&self, sql: &str, needles: &[String]) -> Result<(Vec<String>, bool), AppError> {
        let mut statement = self.connection.prepare(sql).map_err(AppError::database)?;
        let mut rows = statement.query([]).map_err(AppError::database)?;
        let mut ids = Vec::new();
        loop {
            let row = match rows.next() {
                Ok(Some(row)) => row,
                Ok(None) => return Ok((ids, true)),
                // The time budget ran out: keep what was found.
                Err(rusqlite::Error::SqliteFailure(error, _))
                    if error.code == rusqlite::ErrorCode::OperationInterrupted =>
                {
                    return Ok((ids, false));
                }
                Err(error) => return Err(AppError::database(error)),
            };
            let text: String = row.get("text").map_err(AppError::database)?;
            let words: HashSet<String> =
                crate::views::matching::tokens(&text).into_iter().collect();
            if needles.iter().all(|needle| words.contains(needle)) {
                ids.push(row.get("id").map_err(AppError::database)?);
                if ids.len() >= MATCH_LIMIT {
                    return Ok((ids, false));
                }
            }
        }
    }
}

/// One matched message with its chat's facts, as a search result carries them.
const HIT: &str = "SELECT m.id,m.chat_id,c.provider,m.turn_id,m.role,m.ordinal,m.created_at,
     COALESCE(m.search_projection,m.content) AS content,c.title,c.project,c.repository,
     c.created_at AS chat_created_at,c.updated_at AS chat_updated_at,e.evidence,
     a.parent_chat_id,p.title AS parent_title,a.label,a.role AS subagent_role,
     a.depth,a.finished,a.spawn_call_id,CAST(a.chat_id IS NOT NULL AS INTEGER) AS is_subagent,
     CAST(EXISTS(SELECT 1 FROM chat_ssh_copies h WHERE h.chat_id=c.id) AS INTEGER) AS ssh_copy
 FROM messages m JOIN chats c ON c.id=m.chat_id
 LEFT JOIN chat_enrichment e ON e.chat_id=c.id
 LEFT JOIN chat_subagents a ON a.chat_id=c.id
 LEFT JOIN chats p ON p.id=a.parent_chat_id AND p.provider=c.provider
 WHERE m.id=?1";

fn hit(row: &rusqlite::Row<'_>) -> rusqlite::Result<SearchHit> {
    let subagent = (row.get::<_, i64>("is_subagent")? != 0).then(|| -> rusqlite::Result<_> {
        Ok(crate::subagents::Subagent::from(crate::subagents::Row {
            parent_chat_id: row.get("parent_chat_id")?,
            parent_title: row.get("parent_title")?,
            label: row.get("label")?,
            role: row.get("subagent_role")?,
            depth: row.get("depth")?,
            finished: row.get("finished")?,
            spawn_call_id: row.get("spawn_call_id")?,
        }))
    });
    let enrichment = row
        .get::<_, Option<String>>("evidence")?
        .and_then(|json| super::enrichment::decode(&json).ok());
    Ok(SearchHit {
        lookup: None,
        lexical_match: None,
        id: row.get("id")?,
        chat_id: row.get("chat_id")?,
        provider: row.get("provider")?,
        turn_id: row.get("turn_id")?,
        role: row.get("role")?,
        sequence: row.get("ordinal")?,
        content: row.get("content")?,
        created_at: row.get("created_at")?,
        metadata: ChatMetadata {
            enrichment,
            title: row.get("title")?,
            project: row.get("project")?,
            repository: row.get("repository")?,
            created_at: row.get("chat_created_at")?,
            updated_at: row.get("chat_updated_at")?,
            subagent: subagent.transpose()?,
            ssh_copy: row.get::<_, i64>("ssh_copy")? != 0,
        },
        copies: Vec::new(),
        lexical_score: None,
        semantic_score: None,
        fusion_score: 0.0,
        rerank_score: None,
        node_id: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn storage() -> (tempfile::TempDir, Storage) {
        let directory = tempfile::tempdir().unwrap();
        let storage = Storage::open(&directory.path().join("mash.db")).unwrap();
        storage
            .connection
            .execute_batch(
                "INSERT INTO chats(id,provider,title) VALUES ('c1','codex','Log rotation');
                 INSERT INTO chats(id,provider,title) VALUES ('c2','claude-code','Other');
                 INSERT INTO messages(id,chat_id,ordinal,role,content_packed,created_at)
                   VALUES ('m1','c1',0,'user',silver_pack('How do I Rotate the logs?'),1000);
                 INSERT INTO messages(id,chat_id,ordinal,role,content_packed,created_at)
                   VALUES ('m2','c1',1,'assistant',silver_pack('Use logrotate weekly.'),2000);
                 INSERT INTO messages(id,chat_id,ordinal,role,content_packed,search_projection,created_at)
                   VALUES ('m3','c2',0,'assistant',silver_pack('rotate logs in a tool call'),'',3000);
                 INSERT INTO messages(id,chat_id,ordinal,role,content_packed,created_at)
                   VALUES ('m4','c2',1,'user',silver_pack('rotate the logs again'),4000);",
            )
            .unwrap();
        (directory, storage)
    }

    fn scope(provider: &[&str]) -> crate::search_filter::Scope {
        let selection = crate::search_filter::Selection {
            provider: provider.iter().map(|name| name.parse().unwrap()).collect(),
            ..Default::default()
        };
        selection.resolve("local").unwrap()
    }

    fn ids(scan: &Scan) -> Vec<&str> {
        scan.hits.iter().map(|hit| hit.id.as_str()).collect()
    }

    #[test]
    fn every_word_must_appear_and_the_newest_match_comes_first() {
        let (_directory, mut storage) = storage();
        let words = ["rotate".to_owned(), "logs".to_owned()];
        let scan = storage.scan_messages(&words, &scope(&[])).unwrap();
        assert!(scan.complete);
        // m2 says "logrotate", not "rotate"; m3 is hidden from search.
        assert_eq!(ids(&scan), ["m4", "m1"]);
        assert_eq!(scan.hits[1].metadata.title.as_deref(), Some("Log rotation"));
    }

    #[test]
    fn the_scan_keeps_to_the_search_filters() {
        let (_directory, mut storage) = storage();
        let words = ["rotate".to_owned()];
        let scan = storage.scan_messages(&words, &scope(&["codex"])).unwrap();
        // "logrotate" is another word, as it is for indexed search.
        assert_eq!(ids(&scan), ["m1"]);
    }
}

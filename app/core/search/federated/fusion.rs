//! Reciprocal rank fusion of each source's own ranking into one list of chats.
//!
//! A source's scores mean different things (BM25, cosine similarity after
//! reranking, another machine's fused score), so only positions are compared:
//! a chat earns 1 / (K + position) from every source that found it, and the
//! sum orders the merged list. K damps the gap between first and second place
//! so one source's top hit does not drown a chat that several sources agree on.
use super::Source;
use std::collections::HashMap;

/// The usual constant for reciprocal rank fusion.
const K: f64 = 60.0;

/// One merged row per chat, best first. `answered` must already be in
/// `Source` order: the first source to list a chat supplies its row (so this
/// machine's own copy wins over its published one), and ties keep that order.
/// Each row gains `matched_by`, the sources that found it.
pub fn fuse(
    local_node: &str,
    answered: &[(Source, Vec<serde_json::Value>)],
) -> Vec<serde_json::Value> {
    struct Entry {
        row: serde_json::Value,
        score: f64,
        first: (Source, usize),
        matched_by: Vec<Source>,
    }
    let mut entries: Vec<Entry> = Vec::new();
    let mut by_chat: HashMap<(String, String), usize> = HashMap::new();
    for (source, rows) in answered {
        for (position, row) in rows.iter().enumerate() {
            let Some(key) = identity(local_node, row) else {
                continue;
            };
            let gain = 1.0 / (K + f64::from(u32::try_from(position).unwrap_or(u32::MAX)) + 1.0);
            if let Some(&index) = by_chat.get(&key) {
                let entry = &mut entries[index];
                entry.score += gain;
                if !entry.matched_by.contains(source) {
                    entry.matched_by.push(*source);
                }
            } else {
                by_chat.insert(key, entries.len());
                entries.push(Entry {
                    row: row.clone(),
                    score: gain,
                    first: (*source, position),
                    matched_by: vec![*source],
                });
            }
        }
    }
    entries.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.first.cmp(&b.first)));
    entries
        .into_iter()
        .map(|entry| {
            let mut row = entry.row;
            if let serde_json::Value::Object(fields) = &mut row {
                fields.insert(
                    "matched_by".to_owned(),
                    serde_json::to_value(&entry.matched_by).unwrap_or_default(),
                );
            }
            row
        })
        .collect()
}

/// A chat is the same chat on whichever source found it: its machine (this
/// machine when the row names none) and its ID. A row without an ID is skipped.
fn identity(local_node: &str, row: &serde_json::Value) -> Option<(String, String)> {
    let chat = row.get("chat_id")?.as_str()?.to_owned();
    let node = row
        .get("node_id")
        .and_then(serde_json::Value::as_str)
        .filter(|node| !node.is_empty())
        .unwrap_or(local_node)
        .to_owned();
    Some((node, chat))
}

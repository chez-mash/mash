//! What a call returned or opened, as compact references rather than content:
//! the records, chats, messages and attachments in its response, in the order
//! the response lists them.
//!
//! The response is walked as JSON, so one reader serves the CLI's and the MCP
//! server's shapes alike. A walk stops at the first object that names an item,
//! so the evidence a record cites, or the messages inside a chat hit, are not
//! counted as things the call returned.
use serde_json::{Map, Value};

/// The most references kept for one call; the rest are counted, not kept.
pub(super) const CAP: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Reference {
    pub(super) kind: &'static str,
    /// For a record: decision, preference or note.
    pub(super) record_kind: Option<String>,
    pub(super) id: String,
    /// The machine the item came from, when the response says.
    pub(super) node: Option<String>,
}

/// The references in `response`, up to [`CAP`], and how many there were.
pub(super) fn extract(response: &Value) -> (Vec<Reference>, usize) {
    let mut found = Vec::new();
    let mut total = 0;
    let mut seen = std::collections::HashSet::new();
    walk(response, &mut |reference| {
        if seen.insert((reference.kind, reference.id.clone(), reference.node.clone())) {
            total += 1;
            if found.len() < CAP {
                found.push(reference);
            }
        }
    });
    (found, total)
}

fn walk(value: &Value, emit: &mut impl FnMut(Reference)) {
    match value {
        Value::Object(fields) => match identify(fields) {
            Some(reference) => emit(reference),
            None => fields.values().for_each(|value| walk(value, emit)),
        },
        Value::Array(items) => items.iter().for_each(|item| walk(item, emit)),
        _ => {}
    }
}

fn text<'a>(fields: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    fields.get(key).and_then(Value::as_str)
}

fn identify(fields: &Map<String, Value>) -> Option<Reference> {
    let node = text(fields, "node_id").map(str::to_owned);
    if let Some(id) = text(fields, "id").filter(|id| id.starts_with("record/")) {
        return Some(Reference {
            kind: "record",
            record_kind: record_kind(fields),
            id: id.to_owned(),
            node,
        });
    }
    if let Some(id) = text(fields, "attachment_id") {
        return Some(Reference {
            kind: "attachment",
            record_kind: None,
            id: id.to_owned(),
            node,
        });
    }
    if let Some(id) = text(fields, "message_id") {
        return Some(Reference {
            kind: "message",
            record_kind: None,
            id: id.to_owned(),
            node,
        });
    }
    text(fields, "chat_id").map(|id| Reference {
        kind: "chat",
        record_kind: None,
        id: id.to_owned(),
        node,
    })
}

/// A record's kind, from its current revision when the response lists
/// revisions, or from the record itself when it is flat.
fn record_kind(fields: &Map<String, Value>) -> Option<String> {
    let current = fields.get("current_revision").and_then(Value::as_i64);
    let revisions = fields.get("revisions").and_then(Value::as_array);
    let revision = revisions.and_then(|revisions| {
        revisions
            .iter()
            .find(|revision| revision.get("revision").and_then(Value::as_i64) == current)
            .or_else(|| revisions.first())
    });
    let from_revision = revision
        .and_then(|revision| revision.pointer("/body/details/kind"))
        .and_then(Value::as_str);
    from_revision
        .or_else(|| text(fields, "kind"))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_carry_their_kind_and_not_their_citations() {
        let response = serde_json::json!({"records": [
            {"id": "record/a/1", "current_revision": 2, "revisions": [
                {"revision": 2, "body": {"details": {"kind": "decision"}},
                 "evidence": [{"chat_id": "cited", "message_id": "m"}]},
                {"revision": 1, "body": {"details": {"kind": "preference"}}}
            ]},
            {"id": "record/a/2", "node_id": "xenia", "kind": "preference"}
        ]});
        let (references, total) = extract(&response);
        assert_eq!(total, 2);
        assert_eq!(references[0].record_kind.as_deref(), Some("decision"));
        assert_eq!(references[1].record_kind.as_deref(), Some("preference"));
        assert_eq!(references[1].node.as_deref(), Some("xenia"));
        assert!(
            references
                .iter()
                .all(|reference| reference.kind == "record")
        );
    }

    #[test]
    fn chat_hits_count_once_whatever_their_matches_hold() {
        let response = serde_json::json!({"query": "q", "results": [
            {"chat_id": "a", "node_id": "laptop", "matches": [{"message_id": "m1"}]},
            {"chat_id": "b", "matches": []},
            {"chat_id": "a", "node_id": "laptop"}
        ]});
        let (references, total) = extract(&response);
        let ids: Vec<&str> = references
            .iter()
            .map(|reference| reference.id.as_str())
            .collect();
        assert_eq!(ids, ["a", "b"], "a repeated chat counts once");
        assert_eq!(total, 2);
        assert!(references.iter().all(|reference| reference.kind == "chat"));
    }

    #[test]
    fn only_the_cap_is_kept_but_all_are_counted() {
        let results: Vec<Value> = (0..250)
            .map(|index| serde_json::json!({"message_id": format!("m{index}"), "chat_id": "c"}))
            .collect();
        let (references, total) = extract(&serde_json::json!({"results": results}));
        assert_eq!(references.len(), CAP);
        assert_eq!(total, 250);
        assert_eq!(references[0].kind, "message");
    }

    #[test]
    fn a_response_without_items_has_no_references() {
        let (references, total) = extract(&serde_json::json!({"status": "ok", "count": 3}));
        assert_eq!(references, Vec::new());
        assert_eq!(total, 0);
    }
}

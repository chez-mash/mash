//! The subagent marker on the wire: what newer and older peers make of it.
use super::super::{
    Chat, DEFAULT_MAX_DECODED_BYTES, Enrichment, NativeRecord, decode_publication,
    encode_publication, member_digest, native_record,
};
use crate::shared::protocol::{self, Event};
use crate::silver::subagent::Subagent;
use prost::Message as _;

/// `Chat` as releases before published markers define it (tags 1 to 8).
#[derive(Clone, PartialEq, prost::Message)]
struct OlderChat {
    #[prost(message, optional, tag = "1")]
    enrichment: Option<Enrichment>,
    #[prost(string, tag = "2")]
    id: String,
    #[prost(string, tag = "3")]
    provider: String,
    #[prost(string, optional, tag = "4")]
    title: Option<String>,
    #[prost(string, optional, tag = "5")]
    project: Option<String>,
    #[prost(string, optional, tag = "6")]
    repository: Option<String>,
    #[prost(int64, optional, tag = "7")]
    created_at: Option<i64>,
    #[prost(int64, optional, tag = "8")]
    updated_at: Option<i64>,
}

fn marker() -> Subagent {
    Subagent {
        parent_chat_id: Some("parent".into()),
        label: Some("Tidy the docs".into()),
        role: Some("worker".into()),
        depth: Some(1),
        finished: Some(false),
        spawn_call_id: Some("call-1".into()),
    }
}

fn record(subagent: Option<Subagent>) -> protocol::ChatRecord {
    protocol::ChatRecord {
        enrichment: None,
        id: "child".into(),
        provider: crate::provider::Provider::ClaudeCode,
        title: Some("Child".into()),
        project: Some("/work".into()),
        repository: None,
        created_at: Some(1),
        updated_at: Some(2),
        subagent,
    }
}

fn round_trip(events: &[Event]) -> Vec<Event> {
    let encoded = encode_publication(events, DEFAULT_MAX_DECODED_BYTES).unwrap();
    decode_publication(&encoded.pieces, DEFAULT_MAX_DECODED_BYTES).unwrap()
}

fn chat_bytes(record: &protocol::ChatRecord) -> Vec<u8> {
    let Some(native_record::Body::Chat(chat)) = NativeRecord::from(record).body else {
        panic!("a chat record encodes as a chat");
    };
    chat.encode_to_vec()
}

#[test]
fn chat_records_and_amendments_carry_the_marker_or_none() {
    for subagent in [Some(marker()), Some(Subagent::default()), None] {
        let events = vec![
            Event::Batch(protocol::Batch::Chats {
                records: vec![record(subagent.clone())],
            }),
            Event::Amend(Box::new(protocol::Amendment {
                chat: record(subagent.clone()),
                replaced: Vec::new(),
                messages: 3,
            })),
        ];
        let decoded = round_trip(&events);
        assert_eq!(
            serde_json::to_value(&decoded).unwrap(),
            serde_json::to_value(&events).unwrap()
        );
        let Event::Amend(amendment) = &decoded[1] else {
            panic!("the amendment decodes as one");
        };
        assert_eq!(amendment.chat.subagent, subagent);
    }
}

#[test]
fn older_peers_and_newer_ones_read_each_others_chats() {
    // A newer node to an older server: the marker is an unknown field, dropped.
    let older = OlderChat::decode(chat_bytes(&record(Some(marker()))).as_slice()).unwrap();
    assert_eq!(
        (older.id.as_str(), older.title.as_deref()),
        ("child", Some("Child"))
    );
    // An older node to a newer server: no marker, and an unmarked chat
    // encodes exactly as it did before.
    let unmarked = chat_bytes(&record(None));
    assert_eq!(
        OlderChat::decode(unmarked.as_slice())
            .unwrap()
            .encode_to_vec(),
        unmarked
    );
    let chat = Chat::decode(older.encode_to_vec().as_slice()).unwrap();
    assert_eq!(chat.subagent, None);
    let decoded: protocol::ChatRecord = chat.try_into().unwrap();
    assert_eq!(decoded.subagent, None);
    // The JSON a release before the marker wrote still reads.
    let json = serde_json::to_value(record(None)).unwrap();
    assert!(json.get("subagent").is_none());
    let read: protocol::ChatRecord = serde_json::from_value(json).unwrap();
    assert_eq!(read.subagent, None);
}

#[test]
fn baseline_member_digests_ignore_the_marker() {
    // An older server digests the record it decoded, without the marker; a
    // baseline completes only when both sides agree.
    assert_eq!(
        member_digest(&record(Some(marker()))).unwrap(),
        member_digest(&record(None)).unwrap()
    );
    let mut retitled = record(Some(marker()));
    retitled.title = Some("Other".into());
    assert_ne!(
        member_digest(&retitled).unwrap(),
        member_digest(&record(None)).unwrap()
    );
}

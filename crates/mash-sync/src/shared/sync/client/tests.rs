use super::*;

fn chat(id: &str, updated_at: i64) -> Event {
    Event::Batch(crate::shared::protocol::Batch::Chats {
        records: vec![crate::shared::protocol::ChatRecord {
            enrichment: None,
            id: id.into(),
            provider: crate::provider::Provider::Codex,
            title: Some(id.into()),
            project: None,
            repository: None,
            created_at: Some(1),
            updated_at: Some(updated_at),
            subagent: None,
        }],
    })
}

fn start() -> crate::shared::protocol::Start {
    crate::shared::protocol::Start {
        views_build: None,
        source_silver: None,
        instance: "planner".into(),
        revision: Some(1),
        base_revision: Some(0),
        snapshot: false,
        forgets: Vec::new(),
    }
}

#[test]
fn backfill_keeps_recall_rows_for_the_final_ordered_publication() {
    let owner = Event::Batch(protocol::Batch::Records {
        records: vec![crate::records::transfer::Row::Owner { id: "owner".into() }],
    });
    let plan = plan_backfill(
        start(),
        vec![chat("chat", 1), owner],
        BackfillLimits::default(),
    )
    .unwrap();
    assert_eq!(plan.members.len(), 1);
    let record_batch = plan.record_batch.as_ref().unwrap();
    let rows = wire::decode_publication(
        &plan.batches[0].encoded.pieces,
        wire::DEFAULT_MAX_DECODED_BYTES,
    )
    .unwrap();
    assert!(
        rows.iter()
            .all(|event| !matches!(event, Event::Batch(protocol::Batch::Records { .. })))
    );
    let records =
        wire::decode_publication(&record_batch.pieces, wire::DEFAULT_MAX_DECODED_BYTES).unwrap();
    assert!(
        matches!(&records[0], Event::Batch(protocol::Batch::Records { records }) if records.len()==1)
    );
    let expected = wire::encode_publication(&records, wire::DEFAULT_MAX_DECODED_BYTES).unwrap();
    assert_eq!(record_batch.manifest_digest, expected.manifest_digest);
    assert_eq!(record_batch.decoded_bytes, expected.decoded_bytes);
}

#[test]
fn backfill_rejects_an_oversized_final_record_batch_during_planning() {
    let events = || {
        vec![
            chat("chat", 1),
            Event::Batch(protocol::Batch::Records {
                records: vec![crate::records::transfer::Row::Owner {
                    id: "o".repeat(128),
                }],
            }),
        ]
    };
    let failure = plan_backfill_bounded(start(), events(), BackfillLimits::default(), 32)
        .err()
        .unwrap();
    assert!(failure.error.message.contains(wire::BATCH_TOO_LARGE));
    let first = plan_backfill_bounded(start(), events(), BackfillLimits::default(), 1024).unwrap();
    let retry = plan_backfill_bounded(start(), events(), BackfillLimits::default(), 1024).unwrap();
    assert_eq!(
        first.record_batch.unwrap().manifest_digest,
        retry.record_batch.unwrap().manifest_digest
    );
    assert!(
        plan_backfill(start(), vec![chat("chat", 1)], BackfillLimits::default())
            .unwrap()
            .record_batch
            .is_none()
    );
}

#[test]
fn the_planner_splits_whole_chats_newest_first() {
    let events = || vec![chat("old", 10), chat("new", 30), chat("mid", 20)];
    let limits = BackfillLimits {
        chats: 2,
        decoded_bytes: 1024 * 1024,
    };
    let plan = plan_backfill(start(), events(), limits).unwrap();
    assert_eq!(plan.batches.len(), 2, "three chats at two per batch");
    // Membership is identified by chat id; recency decides order.
    assert_eq!(
        plan.batches[0]
            .members
            .iter()
            .map(|member| member.chat_id.as_str())
            .collect::<Vec<_>>(),
        vec!["mid", "new"],
        "the newest two conversations fill the first batch"
    );
    assert_eq!(
        plan.batches[1].members[0].chat_id, "old",
        "the oldest conversation is sent last"
    );
    assert_eq!(plan.members.len(), 3, "manifest covers every conversation");

    // Planning is deterministic: retries produce identical batch digests.
    let again = plan_backfill(start(), events(), limits).unwrap();
    assert_eq!(
        plan.batches
            .iter()
            .map(PlannedBatch::digest)
            .collect::<Vec<_>>(),
        again
            .batches
            .iter()
            .map(PlannedBatch::digest)
            .collect::<Vec<_>>()
    );
}

#[test]
fn the_planner_never_splits_a_chat_across_batches() {
    // Two chats forced into separate batches by the byte target.
    let events = vec![
        Event::Batch(crate::shared::protocol::Batch::Chats {
            records: vec![crate::shared::protocol::ChatRecord {
                enrichment: None,
                id: "big".into(),
                provider: crate::provider::Provider::Codex,
                title: Some("big".into()),
                project: None,
                repository: None,
                created_at: Some(1),
                updated_at: Some(2),
                subagent: None,
            }],
        }),
        Event::Batch(crate::shared::protocol::Batch::Messages {
            records: vec![crate::shared::protocol::MessageRecord {
                attachments: Vec::new(),
                id: "big-1".into(),
                chat_id: "big".into(),
                ordinal: 0,
                context: crate::message_context::MessageContext::Linear,
                role: crate::role::Role::User,
                content: "x".repeat(2048),
                projection: "x".repeat(2048),
                created_at: Some(1),
                generation: None,
                vector: None,
                tools: crate::tools::Activity::default(),
            }],
        }),
        chat("small", 1),
    ];
    let plan = plan_backfill(
        start(),
        events,
        BackfillLimits {
            chats: 128,
            decoded_bytes: 1024,
        },
    )
    .unwrap();
    assert_eq!(plan.batches.len(), 2);
    let big = plan
        .batches
        .iter()
        .find(|batch| batch.members.iter().any(|member| member.chat_id == "big"))
        .expect("big conversation present");
    // The chat record and its message travel together.
    let bound = crate::shared::protocol::ChatRecord {
        enrichment: None,
        id: "big".into(),
        provider: crate::provider::Provider::Codex,
        title: Some("big".into()),
        project: None,
        repository: None,
        created_at: Some(1),
        updated_at: Some(2),
        subagent: None,
    };
    assert_eq!(
        big.members[0].digest,
        crate::shared::sync::wire::member_digest(&bound).unwrap()
    );
    assert!(
        big.encoded.decoded_bytes > 2048,
        "message stayed in the batch"
    );
}

/// One publication spool with a chat record.
fn spool(instance: &str) -> tempfile::NamedTempFile {
    use std::io::Write as _;
    let file = tempfile::NamedTempFile::new().unwrap();
    let mut writer = std::io::BufWriter::new(file.as_file());
    let start = crate::shared::protocol::Start {
        views_build: None,
        source_silver: None,
        instance: instance.into(),
        revision: Some(1),
        base_revision: None,
        snapshot: true,
        forgets: Vec::new(),
    };
    let record = crate::shared::protocol::ChatRecord {
        enrichment: None,
        id: "reuse-chat".into(),
        provider: crate::provider::Provider::Codex,
        title: Some("reuse".into()),
        project: None,
        repository: None,
        created_at: Some(1),
        updated_at: Some(2),
        subagent: None,
    };
    for event in [
        Event::Start(start),
        Event::Batch(crate::shared::protocol::Batch::Chats {
            records: vec![record],
        }),
        Event::Finish,
    ] {
        serde_json::to_writer(&mut writer, &event).unwrap();
        writer.write_all(b"\n").unwrap();
    }
    writer.flush().unwrap();
    drop(writer);
    file
}

fn server_replies(
    frame: &wire::ClientFrame,
    publications: &std::sync::atomic::AtomicUsize,
) -> Vec<wire::server_frame::Body> {
    match &frame.body {
        Some(client_frame::Body::Hello(_)) => {
            vec![server_frame::Body::Welcome(wire::Welcome {
                protocol_version: wire::PROTOCOL_VERSION,
                session_id: "session".into(),
                credit_bytes: wire::DEFAULT_CREDIT_BYTES,
                max_envelope_bytes: u32::try_from(wire::MAX_ENVELOPE_BYTES).unwrap(),
                max_decoded_bytes: u64::try_from(wire::DEFAULT_MAX_DECODED_BYTES).unwrap(),
                heartbeat_ms: 30_000,
                retained_revision: None,
                views_fingerprint: "generation".into(),
                incomplete_retention_ms: 24 * 60 * 60 * 1000,
            })]
        }
        Some(client_frame::Body::Resume(_)) => {
            vec![server_frame::Body::Outcome(wire::Outcome {
                state: wire::OutcomeState::Unknown as i32,
                commit_state: wire::CommitState::NotCommitted as i32,
                checkpoint_sequence: 0,
                revision: None,
                receipt_digest: String::new(),
            })]
        }
        Some(client_frame::Body::BeginPublication(_)) => {
            vec![server_frame::Body::Accepted(wire::Accepted {
                credit_bytes: 0,
            })]
        }
        Some(client_frame::Body::Chunk(chunk)) => vec![
            server_frame::Body::Checkpoint(wire::Checkpoint {
                sequence: chunk.sequence,
                decoded_offset: chunk.decoded_offset + chunk.decoded_length,
            }),
            server_frame::Body::Credit(wire::Credit {
                additional_bytes: u32::try_from(chunk.data.len()).unwrap_or(u32::MAX),
            }),
        ],
        Some(client_frame::Body::FinishUpload(_)) => {
            publications.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            vec![server_frame::Body::Published(wire::Published {
                revision: 1,
                views_current: true,
                coverage_through: None,
            })]
        }
        _ => Vec::new(),
    }
}

type Counters = (
    std::sync::Arc<std::sync::atomic::AtomicUsize>,
    std::sync::Arc<std::sync::atomic::AtomicUsize>,
);

/// Fake `/v1/sync` server: counts connections, serves any number of
/// publications per connection, and optionally closes after the first.
async fn start_server(close_after_first: bool) -> (String, Counters, tokio::task::JoinHandle<()>) {
    use axum::Router;
    use axum::extract::State;
    use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
    use futures_util::{SinkExt, StreamExt};

    async fn serve(mut socket: WebSocket, counters: Counters, close_after_first: bool) {
        let (connections, publications) = &counters;
        connections.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        while let Some(Ok(message)) = socket.next().await {
            let WsMessage::Binary(bytes) = message else {
                continue;
            };
            let Ok(frame) = wire::decode_client(&bytes) else {
                break;
            };
            let finished = matches!(frame.body, Some(client_frame::Body::FinishUpload(_)));
            for body in server_replies(&frame, publications) {
                let reply = ServerFrame {
                    operation_id: frame.operation_id.clone(),
                    body: Some(body),
                };
                if socket
                    .send(WsMessage::Binary(
                        wire::encode_server(&reply).unwrap().into(),
                    ))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            if close_after_first && finished {
                let _ = socket.close().await;
                return;
            }
        }
    }

    let counters: Counters = (
        std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let state = counters.clone();
    let task = tokio::spawn(async move {
        let router = Router::new()
            .route(
                "/v1/sync",
                axum::routing::get(
                    move |upgrade: WebSocketUpgrade, State(counters): State<Counters>| async move {
                        upgrade.on_upgrade(move |socket| serve(socket, counters, close_after_first))
                    },
                ),
            )
            .with_state(state);
        axum::serve(listener, router).await.unwrap();
    });
    (format!("http://{address}"), counters, task)
}

fn session_config(api_url: String) -> SharedConfig {
    SharedConfig {
        group_id: "group".into(),
        node_id: "node".into(),
        api_url,
        token_env: "unused".into(),
        publication_redaction: crate::redaction::Policy::new(false),
        unshared: Vec::new(),
    }
}

#[tokio::test]
async fn publications_reuse_one_publisher_session() {
    let (url, counters, server) = start_server(false).await;
    let config = session_config(url);
    let file = spool("reuse-instance");
    for _ in 0..3 {
        crate::shared::sync::client::upload_with_token(&config, &file, "test-token".to_owned())
            .await
            .expect("publication");
    }
    assert_eq!(
        counters.0.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "three publications share one handshake"
    );
    assert_eq!(
        counters.1.load(std::sync::atomic::Ordering::SeqCst),
        3,
        "each publication reached the server"
    );
    server.abort();
}

#[tokio::test]
async fn a_closed_session_is_replaced_on_the_next_publication() {
    let (url, counters, server) = start_server(true).await;
    let config = session_config(url);
    let file = spool("reuse-instance");
    crate::shared::sync::client::upload_with_token(&config, &file, "test-token".to_owned())
        .await
        .expect("first publication");
    crate::shared::sync::client::upload_with_token(&config, &file, "test-token".to_owned())
        .await
        .expect("second publication");
    assert_eq!(
        counters.0.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "the server closed the first session, so the second reconnects"
    );
    assert_eq!(counters.1.load(std::sync::atomic::Ordering::SeqCst), 2);
    server.abort();
}

#[test]
fn websocket_endpoint_is_derived_without_credentials() {
    assert_eq!(
        endpoint("https://example.test/mash/").unwrap(),
        "wss://example.test/mash/v1/sync"
    );
    assert_eq!(
        endpoint("http://127.0.0.1:8787").unwrap(),
        "ws://127.0.0.1:8787/v1/sync"
    );
    assert!(endpoint("ftp://example.test").is_err());
}

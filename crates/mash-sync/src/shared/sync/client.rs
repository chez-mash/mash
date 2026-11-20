//! Native publication client for the authenticated `/v1/sync` WebSocket.
//!
//! Preparation still uses the existing consistent silver snapshot and NDJSON
//! spool. This module turns that sealed batch into typed Protobuf records,
//! compresses every piece with zstd, and resumes from the server's durable
//! checkpoint instead of restarting a whole upload.
use super::wire::{
    self, AttachmentStored, BeginAttachment, BeginBackfill, BeginPublication, ClientFrame,
    FinishBackfill, FinishUpload, Hello, PublicationMode, PublicationStart, Resume, ServerFrame,
    client_frame, server_frame,
};
use crate::AppError;
use crate::config::SharedConfig;
use crate::shared::protocol::{self, Event};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{self, Message, http::HeaderValue};

mod plan;
pub use plan::*;

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(30);
const OPERATION_TIMEOUT: Duration = Duration::from_mins(15);

fn endpoint(api_url: &str) -> Result<String, AppError> {
    let url = api_url.trim_end_matches('/');
    let websocket = if let Some(rest) = url.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = url.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        return Err(AppError::configuration(
            "shared api_url must use http or https for WebSocket synchronization",
        ));
    };
    Ok(format!("{websocket}/v1/sync"))
}

fn token(config: &SharedConfig) -> Result<String, AppError> {
    let token = config.token().ok_or_else(|| {
        AppError::configuration(format!("set {} to the node's API token", config.token_env))
    })?;
    // Header safety only: the server remains authoritative for token length and
    // compares the credential against its configured node tokens.
    if token.is_empty() || token.len() > 4096 || !token.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(AppError::configuration(
            "API tokens require 1..4096 printable ASCII bytes",
        ));
    }
    Ok(token)
}

fn request(
    config: &SharedConfig,
    token: &str,
) -> Result<tungstenite::handshake::client::Request, AppError> {
    let mut request = endpoint(&config.api_url)?
        .into_client_request()
        .map_err(|_| AppError::shared("invalid shared synchronization endpoint"))?;
    let headers = request.headers_mut();
    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_| AppError::configuration("API token is not a valid header value"))?,
    );
    for (name, value) in [
        ("x-mash-group", &config.group_id),
        ("x-mash-node", &config.node_id),
    ] {
        headers.insert(
            name,
            HeaderValue::from_str(value).map_err(|_| {
                AppError::configuration("shared node identity is not a header value")
            })?,
        );
    }
    Ok(request)
}

/// Read the sealed publication spool produced by the existing preparation path.
fn read_events(file: &tempfile::NamedTempFile) -> Result<(protocol::Start, Vec<Event>), AppError> {
    use std::io::BufRead;
    let lines = std::io::BufReader::new(file.reopen().map_err(AppError::io)?).lines();
    let mut start = None;
    let mut events = Vec::new();
    let mut finished = false;
    for line in lines {
        let line = line.map_err(|_| AppError::usage("invalid index stream"))?;
        let event: Event =
            serde_json::from_str(&line).map_err(|_| AppError::usage("invalid index event"))?;
        match event {
            Event::Start(input) => {
                if start.replace(input).is_some() || !events.is_empty() {
                    return Err(AppError::usage("duplicate index start"));
                }
            }
            Event::Finish => finished = true,
            event => {
                if start.is_none() || finished {
                    return Err(AppError::usage("index stream metadata is out of order"));
                }
                events.push(event);
            }
        }
    }
    let start = start.ok_or_else(|| AppError::usage("empty index stream"))?;
    if !finished {
        return Err(AppError::usage("index stream did not finish"));
    }
    Ok((start, events))
}

async fn send(
    sink: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    frame: &ClientFrame,
) -> Result<(), AppError> {
    let bytes =
        wire::encode_client(frame).map_err(|_| AppError::shared("sync frame is too large"))?;
    sink.send(Message::Binary(bytes.into()))
        .await
        .map_err(|_| AppError::shared("shared synchronization connection failed"))
}

async fn receive(
    stream: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
) -> Result<ServerFrame, AppError> {
    loop {
        let message = tokio::time::timeout(OPERATION_TIMEOUT, stream.next())
            .await
            .map_err(|_| AppError::shared("shared synchronization timed out"))?
            .ok_or_else(|| AppError::shared("shared synchronization closed unexpectedly"))?
            .map_err(|_| AppError::shared("shared synchronization connection failed"))?;
        match message {
            Message::Binary(bytes) => {
                return wire::decode_server(&bytes)
                    .map_err(|_| AppError::shared("invalid synchronization response"));
            }
            Message::Close(_) => {
                return Err(AppError::shared(
                    "shared synchronization closed before acknowledgement",
                ));
            }
            // Tungstenite answers pings; ignore unsolicited text and pongs.
            Message::Text(_) | Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {}
        }
    }
}

fn outcome_error(failed: &wire::Failed) -> AppError {
    let retryable = failed.retryable;
    let message = format!(
        "shared synchronization {} failed: {} ({}){}",
        failed.phase,
        failed.code,
        failed.message,
        if failed.recovery_action.is_empty() {
            String::new()
        } else {
            format!("; {}", failed.recovery_action)
        }
    );
    let mut error = AppError::shared(message);
    error.error.retryable = retryable;
    // A rejected publication keeps the server's meaning (a usage error stays
    // one); older servers send no kind.
    if let Some(kind) = crate::ErrorKind::parse(&failed.kind) {
        error.error.kind = kind;
    }
    error
}

/// Upload one sealed publication batch through the typed WebSocket transport.
pub async fn upload(
    config: &SharedConfig,
    file: &tempfile::NamedTempFile,
) -> Result<protocol::Indexed, AppError> {
    upload_with_token(config, file, token(config)?).await
}

/// Send a prepared batch without rereading or re-encoding its canonical spool.
pub async fn upload_encoded(
    config: &SharedConfig,
    start: &protocol::Start,
    encoded: &wire::EncodedBatch,
) -> Result<protocol::Indexed, AppError> {
    upload_encoded_with_token(config, token(config)?, start, encoded).await
}

/// Publisher sessions are reused across publications from the same process and
/// destination. Watch and MCP publish many small batches; the handshake would
/// otherwise cost more than the saved bytes.
const SESSION_IDLE: Duration = Duration::from_secs(30);

type SessionSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;
type SessionStream = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;
type SessionSlot = std::sync::Arc<tokio::sync::Mutex<Option<PublisherSession>>>;

struct PublisherSession {
    sink: SessionSink,
    stream: SessionStream,
    /// Remaining encoded-byte credit for the next operation.
    credit: usize,
    last_used: tokio::time::Instant,
}

static SESSIONS: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, SessionSlot>>,
> = std::sync::OnceLock::new();

fn session_slots() -> &'static std::sync::Mutex<std::collections::HashMap<String, SessionSlot>> {
    SESSIONS.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// One session per endpoint, node and database instance. A destination or
/// identity change must negotiate again rather than reuse another's stream.
fn session_key(config: &SharedConfig, instance: &str) -> String {
    format!(
        "{}\0{}\0{}\0{instance}",
        config.api_url.trim_end_matches('/'),
        config.group_id,
        config.node_id
    )
}

fn session_slot(key: &str) -> SessionSlot {
    let mut slots = session_slots()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    slots
        .entry(key.to_owned())
        .or_insert_with(|| std::sync::Arc::new(tokio::sync::Mutex::new(None)))
        .clone()
}

impl PublisherSession {
    /// Connect and negotiate; the session is reusable until it idles out or an
    /// operation fails on it.
    async fn connect(config: &SharedConfig, token: &str, instance: &str) -> Result<Self, AppError> {
        let request = request(config, token)?;
        let (socket, _) =
            tokio::time::timeout(HANDSHAKE_TIMEOUT, tokio_tungstenite::connect_async(request))
                .await
                .map_err(|_| AppError::shared("shared synchronization handshake timed out"))?
                .map_err(|_| {
                    AppError::shared("could not connect to the shared synchronization endpoint")
                })?;
        let (mut sink, mut stream) = socket.split();
        send(
            &mut sink,
            &ClientFrame {
                operation_id: String::new(),
                body: Some(client_frame::Body::Hello(Hello {
                    protocol_version: wire::PROTOCOL_VERSION,
                    database_instance: instance.to_owned(),
                    zstd_publication_data: true,
                    max_decoded_bytes: wire::DEFAULT_MAX_DECODED_BYTES as u64,
                })),
            },
        )
        .await?;
        let welcome = match receive(&mut stream).await?.body {
            Some(server_frame::Body::Welcome(welcome))
                if welcome.protocol_version == wire::PROTOCOL_VERSION
                    && welcome.max_envelope_bytes > 0 =>
            {
                welcome
            }
            Some(server_frame::Body::Rejected(failed) | server_frame::Body::Failed(failed)) => {
                return Err(outcome_error(&failed));
            }
            _ => {
                return Err(AppError::shared(
                    "shared synchronization did not accept this protocol version",
                ));
            }
        };
        Ok(Self {
            sink,
            stream,
            credit: usize::try_from(welcome.credit_bytes).unwrap_or(0),
            last_used: tokio::time::Instant::now(),
        })
    }
}

/// Reuse the live session when it is fresh, otherwise negotiate a new one.
async fn ensure_session<'a>(
    slot: &'a mut Option<PublisherSession>,
    config: &SharedConfig,
    token: &str,
    instance: &str,
) -> Result<&'a mut PublisherSession, AppError> {
    if slot
        .as_ref()
        .is_some_and(|session| session.last_used.elapsed() > SESSION_IDLE)
    {
        *slot = None;
    }
    if slot.is_none() {
        *slot = Some(PublisherSession::connect(config, token, instance).await?);
    }
    slot.as_mut()
        .ok_or_else(|| AppError::internal("publisher session disappeared"))
}

/// Same publisher with an explicit credential, so tests do not mutate process env.
pub async fn upload_with_token(
    config: &SharedConfig,
    file: &tempfile::NamedTempFile,
    token: String,
) -> Result<protocol::Indexed, AppError> {
    let (start, events) = read_events(file)?;
    let encoded = wire::encode_publication(&events, wire::DEFAULT_MAX_DECODED_BYTES)?;
    if encoded.manifest_digest.is_empty() {
        return Err(AppError::internal("publication manifest is empty"));
    }
    upload_encoded_with_token(config, token, &start, &encoded).await
}

/// Publish an already-encoded batch through the shared publisher session. The
/// spool-reading entry point above and the benchmarks both land here.
pub async fn upload_encoded_with_token(
    config: &SharedConfig,
    token: String,
    start: &protocol::Start,
    encoded: &wire::EncodedBatch,
) -> Result<protocol::Indexed, AppError> {
    let operation_id = scoped_operation_id(
        "publication",
        config,
        &publication_digest(start, &encoded.manifest_digest),
    );
    let slot = session_slot(&session_key(config, &start.instance));
    let mut guard = slot.lock_owned().await;
    let reused = guard.is_some();
    let result = async {
        let session = ensure_session(&mut guard, config, &token, &start.instance).await?;
        publish_on_session(session, &operation_id, start, encoded, None).await
    }
    .await;
    match result {
        Ok(indexed) => Ok(indexed),
        Err(first) => {
            // The cached stream may have been closed by the peer; the operation
            // is resumable, so negotiate once more before surfacing an error.
            // A final rejection is not retried: the resumed operation could
            // only report that it no longer accepts data, hiding the cause.
            *guard = None;
            if !reused || !first.error.retryable {
                return Err(first);
            }
            let retry = async {
                let session = ensure_session(&mut guard, config, &token, &start.instance).await?;
                publish_on_session(session, &operation_id, start, encoded, None).await
            }
            .await;
            if retry.is_err() {
                *guard = None;
            }
            retry
        }
    }
}

/// Run one publication on an established session.
#[expect(
    clippy::too_many_lines,
    reason = "one resumable protocol operation with explicit server outcomes"
)]
async fn publish_on_session(
    session: &mut PublisherSession,
    operation_id: &str,
    start: &protocol::Start,
    encoded: &wire::EncodedBatch,
    backfill: Option<(&str, i64)>,
) -> Result<protocol::Indexed, AppError> {
    let mut checkpoint = 0u64;

    // Ask for persisted state before resending. An unknown operation is normal.
    send(
        &mut session.sink,
        &ClientFrame {
            operation_id: operation_id.to_owned(),
            body: Some(client_frame::Body::Resume(Resume {
                manifest_digest: encoded.manifest_digest.clone(),
            })),
        },
    )
    .await?;
    match receive(&mut session.stream).await?.body {
        Some(server_frame::Body::Outcome(outcome)) => {
            if outcome.state == wire::OutcomeState::Published as i32 {
                // A stored published outcome predates this release's field; keep the
                // historical assumption for resumed operations.
                return Ok(indexed(outcome.revision.unwrap_or_default(), true));
            }
            checkpoint = outcome.checkpoint_sequence;
            if outcome.state == wire::OutcomeState::Applying as i32
                || outcome.state == wire::OutcomeState::ViewsPending as i32
            {
                // The server owns the commit; re-begin with the same operation
                // and send FinishUpload so it can report the durable outcome.
                checkpoint = encoded.pieces.len().saturating_sub(1) as u64;
            }
        }
        Some(server_frame::Body::Credit(grant)) => {
            session.credit = session
                .credit
                .saturating_add(grant.additional_bytes as usize);
        }
        _ => {}
    }

    send(
        &mut session.sink,
        &ClientFrame {
            operation_id: operation_id.to_owned(),
            body: Some(client_frame::Body::BeginPublication(BeginPublication {
                start: Some(PublicationStart::from(start)),
                mode: match backfill {
                    Some(_) => PublicationMode::BackfillBatch as i32,
                    None if start.snapshot => PublicationMode::ReplaceSnapshot as i32,
                    None => PublicationMode::Delta as i32,
                },
                backfill_id: backfill.map_or_else(String::new, |(id, _)| id.to_owned()),
                source_revision: backfill.map_or(start.revision, |(_, revision)| Some(revision)),
                journal_from_exclusive: None,
                journal_through_inclusive: None,
                manifest_digest: encoded.manifest_digest.clone(),
                decoded_bytes: encoded.decoded_bytes,
                record_count: encoded.record_count,
            })),
        },
    )
    .await?;
    loop {
        match receive(&mut session.stream).await?.body {
            Some(server_frame::Body::Accepted(accepted)) => {
                session.credit = session
                    .credit
                    .saturating_add(accepted.credit_bytes as usize);
                break;
            }
            Some(server_frame::Body::Outcome(outcome))
                if outcome.state == wire::OutcomeState::Published as i32 =>
            {
                // A stored published outcome predates this release's field; keep the
                // historical assumption for resumed operations.
                return Ok(indexed(outcome.revision.unwrap_or_default(), true));
            }
            Some(server_frame::Body::Credit(grant)) => {
                session.credit = session
                    .credit
                    .saturating_add(grant.additional_bytes as usize);
            }
            Some(server_frame::Body::Rejected(failed) | server_frame::Body::Failed(failed)) => {
                return Err(outcome_error(&failed));
            }
            Some(_) => {}
            None => return Err(AppError::shared("empty synchronization response")),
        }
    }

    let mut next = usize::try_from(checkpoint)
        .ok()
        .filter(|checkpoint| *checkpoint > 0)
        .map_or(0, |checkpoint| checkpoint + 1);
    if next > encoded.pieces.len() {
        return Err(AppError::shared(
            "server resumption checkpoint exceeds the prepared batch",
        ));
    }
    let mut acknowledged = next.saturating_sub(1) as u64;
    let mut finish_sent = false;
    loop {
        while next < encoded.pieces.len() {
            let piece = &encoded.pieces[next];
            let size = piece.encoded.len();
            if size > session.credit {
                break;
            }
            send(
                &mut session.sink,
                &ClientFrame {
                    operation_id: operation_id.to_owned(),
                    body: Some(client_frame::Body::Chunk(wire::EncodedBatch::chunk(piece))),
                },
            )
            .await?;
            session.credit -= size;
            next += 1;
        }
        if next == encoded.pieces.len() && acknowledged + 1 >= next as u64 && !finish_sent {
            send(
                &mut session.sink,
                &ClientFrame {
                    operation_id: operation_id.to_owned(),
                    body: Some(client_frame::Body::FinishUpload(FinishUpload {
                        manifest_digest: encoded.manifest_digest.clone(),
                    })),
                },
            )
            .await?;
            finish_sent = true;
        }
        match receive(&mut session.stream).await?.body {
            Some(server_frame::Body::Credit(grant)) => {
                session.credit = session
                    .credit
                    .saturating_add(grant.additional_bytes as usize);
            }
            Some(server_frame::Body::Checkpoint(checkpoint)) => {
                acknowledged = acknowledged.max(checkpoint.sequence);
            }
            Some(server_frame::Body::Published(published)) => {
                session.last_used = tokio::time::Instant::now();
                return Ok(indexed(published.revision, published.views_current));
            }
            Some(server_frame::Body::Rejected(failed) | server_frame::Body::Failed(failed)) => {
                return Err(outcome_error(&failed));
            }
            Some(server_frame::Body::Cancelled(_)) => {
                return Err(AppError::shared(
                    "shared synchronization operation was cancelled",
                ));
            }
            Some(server_frame::Body::Outcome(outcome))
                if outcome.state == wire::OutcomeState::Published as i32 =>
            {
                session.last_used = tokio::time::Instant::now();
                // A stored published outcome predates this release's field; keep the
                // historical assumption for resumed operations.
                return Ok(indexed(outcome.revision.unwrap_or_default(), true));
            }
            Some(_) => {}
            None => return Err(AppError::shared("empty synchronization response")),
        }
    }
}

/// Upload one attachment as bounded uncompressed pieces over `/v1/sync`.
pub async fn upload_attachment(
    config: &SharedConfig,
    message_id: &str,
    metadata: &crate::attachments::Metadata,
    bytes: &[u8],
) -> Result<(), AppError> {
    upload_attachment_with_token(config, token(config)?, message_id, metadata, bytes).await
}

/// Same transport with an explicit credential, so tests do not mutate process env.
#[expect(
    clippy::too_many_lines,
    reason = "one handshake, begin, chunk and acknowledgement sequence"
)]
pub async fn upload_attachment_with_token(
    config: &SharedConfig,
    token: String,
    message_id: &str,
    metadata: &crate::attachments::Metadata,
    bytes: &[u8],
) -> Result<(), AppError> {
    let mut pieces = Vec::new();
    let mut offset = 0u64;
    for (index, chunk) in bytes.chunks(wire::NORMAL_BATCH_TARGET.max(1)).enumerate() {
        pieces.push(wire::Piece {
            sequence: index as u64,
            decoded_offset: offset,
            decoded_length: chunk.len() as u64,
            digest: blake3::hash(chunk).to_hex().to_string(),
            encoded: chunk.to_vec(),
            // Attachments are binary and already compressed on disk.
            compression: wire::Compression::None as i32,
        });
        offset = offset.saturating_add(chunk.len() as u64);
    }
    if pieces.is_empty() {
        return Err(AppError::usage("attachment upload is empty"));
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
    let manifest = wire::manifest_digest(bytes.len() as u64, 1, &entries);
    let operation_id = scoped_operation_id("attachment", config, &manifest);
    let request = request(config, &token)?;
    let (socket, _) =
        tokio::time::timeout(HANDSHAKE_TIMEOUT, tokio_tungstenite::connect_async(request))
            .await
            .map_err(|_| AppError::shared("shared synchronization handshake timed out"))?
            .map_err(|_| {
                AppError::shared("could not connect to the shared synchronization endpoint")
            })?;
    let (mut sink, mut stream) = socket.split();
    send(
        &mut sink,
        &ClientFrame {
            operation_id: String::new(),
            body: Some(client_frame::Body::Hello(Hello {
                protocol_version: wire::PROTOCOL_VERSION,
                database_instance: config.node_id.clone(),
                zstd_publication_data: true,
                max_decoded_bytes: wire::DEFAULT_MAX_DECODED_BYTES as u64,
            })),
        },
    )
    .await?;
    let welcome = match stream.next().await {
        Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(bytes))) => {
            match wire::decode_server(&bytes)?.body {
                Some(server_frame::Body::Welcome(welcome)) => welcome,
                Some(server_frame::Body::Rejected(failed) | server_frame::Body::Failed(failed)) => {
                    return Err(outcome_error(&failed));
                }
                _ => {
                    return Err(AppError::shared(
                        "shared synchronization did not accept this protocol version",
                    ));
                }
            }
        }
        _ => return Err(AppError::shared("shared synchronization handshake failed")),
    };
    let mut credit = usize::try_from(welcome.credit_bytes).unwrap_or(0);
    send(
        &mut sink,
        &ClientFrame {
            operation_id: operation_id.clone(),
            body: Some(client_frame::Body::BeginAttachment(BeginAttachment {
                message_id: message_id.to_owned(),
                attachment: Some(wire::Attachment::from(metadata)),
                manifest_digest: manifest.clone(),
                decoded_bytes: bytes.len() as u64,
            })),
        },
    )
    .await?;
    let mut next = 0usize;
    loop {
        match receive(&mut stream).await?.body {
            Some(server_frame::Body::Accepted(accepted)) => {
                credit = credit.saturating_add(accepted.credit_bytes as usize);
                break;
            }
            Some(server_frame::Body::Checkpoint(checkpoint)) => {
                next = usize::try_from(checkpoint.sequence)
                    .unwrap_or(usize::MAX)
                    .saturating_add(1);
            }
            Some(server_frame::Body::Outcome(outcome))
                if outcome.state == wire::OutcomeState::Published as i32 =>
            {
                return Ok(());
            }
            Some(server_frame::Body::Credit(grant)) => {
                credit = credit.saturating_add(grant.additional_bytes as usize);
            }
            Some(server_frame::Body::Rejected(failed) | server_frame::Body::Failed(failed)) => {
                return Err(outcome_error(&failed));
            }
            Some(_) => {}
            None => return Err(AppError::shared("empty synchronization response")),
        }
    }
    let mut finish_sent = false;
    loop {
        while next < pieces.len() {
            let piece = &pieces[next];
            if piece.encoded.len() > credit {
                break;
            }
            let chunk = wire::Chunk {
                sequence: piece.sequence,
                decoded_offset: piece.decoded_offset,
                compression: wire::Compression::None as i32,
                decoded_length: piece.decoded_length,
                digest: piece.digest.clone(),
                data: piece.encoded.clone(),
            };
            send(
                &mut sink,
                &ClientFrame {
                    operation_id: operation_id.clone(),
                    body: Some(client_frame::Body::Chunk(chunk)),
                },
            )
            .await?;
            credit -= piece.encoded.len();
            next += 1;
        }
        if next == pieces.len() && !finish_sent {
            send(
                &mut sink,
                &ClientFrame {
                    operation_id: operation_id.clone(),
                    body: Some(client_frame::Body::FinishUpload(FinishUpload {
                        manifest_digest: manifest.clone(),
                    })),
                },
            )
            .await?;
            finish_sent = true;
        }
        match receive(&mut stream).await?.body {
            Some(server_frame::Body::AttachmentStored(AttachmentStored { .. })) => return Ok(()),
            Some(server_frame::Body::Credit(grant)) => {
                credit = credit.saturating_add(grant.additional_bytes as usize);
            }
            Some(server_frame::Body::Outcome(outcome))
                if outcome.state == wire::OutcomeState::Published as i32 =>
            {
                return Ok(());
            }
            Some(server_frame::Body::Rejected(failed) | server_frame::Body::Failed(failed)) => {
                return Err(outcome_error(&failed));
            }
            Some(_) => {}
            None => return Err(AppError::shared("empty synchronization response")),
        }
    }
}

/// A publication's identity: its records (the manifest) and its start
/// metadata (revision, base revision, source certificate), which the
/// manifest does not cover. Without the start, re-sending the same records
/// at another revision would replay the earlier receipt unvalidated.
fn publication_digest(start: &protocol::Start, manifest: &str) -> String {
    let mut digest = blake3::Hasher::new();
    digest.update(b"mash-publication-v1\0");
    digest.update(manifest.as_bytes());
    digest.update(b"\0");
    digest.update(&serde_json::to_vec(start).unwrap_or_default());
    digest.finalize().to_hex().to_string()
}

/// Operation identity is scoped to the publishing node. Two nodes may send
/// identical bytes, and the transfer catalog must not confuse their operations.
pub fn scoped_operation_id(kind: &str, config: &SharedConfig, digest: &str) -> String {
    // The catalog is service-wide, so the group belongs to the identity:
    // identical bytes published into another group must not reuse (or be
    // rejected by) the first group's operation.
    format!("{kind}:{}:{}:{digest}", config.group_id, config.node_id)
}

/// Coordinator with the process credential, used by publication.
pub async fn upload_backfill_plan_with_config(
    config: &SharedConfig,
    plan: &BackfillPlan,
    backfill_id: &str,
    resume: BackfillResume<'_>,
    on_batch: impl FnMut(&str, i64) -> Result<(), AppError>,
) -> Result<i64, AppError> {
    upload_backfill_plan(
        config,
        token(config)?,
        plan,
        backfill_id,
        resume,
        None,
        backfill_lanes(),
        on_batch,
    )
    .await
}

/// Baseline batches in flight at once. The server's mutation capacity is the
/// real ceiling; additional lanes only add waiters, so the default tracks a
/// four-permit deployment. `MASH_BACKFILL_LANES` overrides it for measurement.
const DEFAULT_BACKFILL_LANES: usize = 4;
const MAX_BACKFILL_LANES: usize = 8;

pub fn backfill_lanes() -> usize {
    std::env::var("MASH_BACKFILL_LANES")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_BACKFILL_LANES)
        .clamp(1, MAX_BACKFILL_LANES)
}

/// Send a planned baseline: manifest, then every batch that is not already
/// recorded as complete, then completion.
///
/// Batches are independent, so they are spread across sessions and committed
/// concurrently; each session carries one active operation.
/// Returns the greatest acknowledged server revision. Baseline coverage does
/// not prove node-wide views readiness; callers must read freshness for that.
#[expect(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "one baseline session: manifest, per-batch commits, completion"
)]
pub async fn upload_backfill_plan(
    config: &SharedConfig,
    token: String,
    plan: &BackfillPlan,
    backfill_id: &str,
    resume: BackfillResume<'_>,
    progress: Option<tokio::sync::mpsc::UnboundedSender<BackfillProgress>>,
    lanes: usize,
    mut on_batch: impl FnMut(&str, i64) -> Result<(), AppError>,
) -> Result<i64, AppError> {
    let BackfillResume {
        completed,
        last_revision,
    } = resume;
    let key = session_key(config, &plan.start.instance);
    // A connection carries exactly one active operation, so batches pipeline
    // across sessions instead of waiting for each commit before sending the
    // next one. Baseline batches are independent by construction.
    let lanes = lanes
        .clamp(1, MAX_BACKFILL_LANES)
        .min(plan.batches.len().max(1));
    let mut workers = Vec::with_capacity(lanes);
    for lane in 0..lanes {
        let lane_key = if lane == 0 {
            key.clone()
        } else {
            format!("{key}\u{0}lane-{lane}")
        };
        let guard = session_slot(&lane_key).lock_owned().await;
        let batches: Vec<usize> = (lane..plan.batches.len())
            .step_by(lanes)
            .filter(|index| !completed.contains(plan.batches[*index].digest()))
            .collect();
        workers.push((guard, batches));
    }

    // The manifest must exist before any batch references it.
    let manifest = {
        let guard = &mut workers[0].0;
        match ensure_session(guard, config, &token, &plan.start.instance).await {
            Ok(session) => install_backfill_manifest(session, backfill_id, plan, config).await,
            Err(error) => Err(error),
        }
    };
    if let Err(error) = manifest {
        for (guard, _) in &mut workers {
            **guard = None;
        }
        return Err(error);
    }

    let receipts = std::sync::Mutex::new(&mut on_batch);
    let revision = std::sync::atomic::AtomicI64::new(last_revision);
    let total = plan.batches.len();
    let workers =
        futures_util::future::try_join_all(workers.into_iter().map(|(mut guard, indices)| {
            let token = token.clone();
            let progress = progress.clone();
            let receipts = &receipts;
            let revision = &revision;
            async move {
                let result: Result<(), AppError> = async {
                    let session =
                        ensure_session(&mut guard, config, &token, &plan.start.instance).await?;
                    for index in indices {
                        let batch = &plan.batches[index];
                        let operation = scoped_operation_id(
                            "publication",
                            config,
                            &publication_digest(&plan.start, batch.digest()),
                        );
                        let indexed = publish_on_session(
                            session,
                            &operation,
                            &plan.start,
                            &batch.encoded,
                            Some((backfill_id, plan.baseline_revision)),
                        )
                        .await?;
                        revision.fetch_max(indexed.revision, std::sync::atomic::Ordering::Relaxed);
                        {
                            let mut callback = receipts
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            (**callback)(batch.digest(), indexed.revision)?;
                        }
                        if let Some(progress) = &progress {
                            let (acknowledge, wait) = tokio::sync::oneshot::channel();
                            if progress
                                .send(BackfillProgress {
                                    index,
                                    total,
                                    revision: indexed.revision,
                                    acknowledge,
                                })
                                .is_ok()
                            {
                                let _ = wait.await;
                            }
                        }
                    }
                    Ok(())
                }
                .await;
                if result.is_err() {
                    *guard = None;
                }
                result
            }
        }))
        .await;
    workers?;

    let revision = revision.load(std::sync::atomic::Ordering::Relaxed);
    let mut guard = session_slot(&key).lock_owned().await;
    let finish = async {
        let session = ensure_session(&mut guard, config, &token, &plan.start.instance).await?;
        finish_backfill(session, backfill_id, plan).await
    }
    .await;
    if finish.is_err() {
        *guard = None;
    }
    finish?;
    Ok(revision)
}

async fn install_backfill_manifest(
    session: &mut PublisherSession,
    backfill_id: &str,
    plan: &BackfillPlan,
    config: &SharedConfig,
) -> Result<(), AppError> {
    let operation = scoped_operation_id("backfill-manifest", config, &plan.manifest_digest);
    send(
        &mut session.sink,
        &ClientFrame {
            operation_id: operation.clone(),
            body: Some(client_frame::Body::BeginBackfill(BeginBackfill {
                backfill_id: backfill_id.to_owned(),
                database_instance: plan.start.instance.clone(),
                baseline_revision: plan.baseline_revision,
                views_fingerprint: views_fingerprint(&plan.start),
                manifest_digest: plan.manifest_digest.clone(),
                total_chats: plan.members.len() as u64,
                decoded_bytes: plan.manifest_bytes,
            })),
        },
    )
    .await?;
    let mut credit = session.credit;
    expect_accepted(&mut session.stream, &mut credit).await?;
    session.credit = credit;
    send(
        &mut session.sink,
        &ClientFrame {
            operation_id: operation.clone(),
            body: Some(client_frame::Body::Chunk(wire::EncodedBatch::chunk(
                &plan.manifest_piece,
            ))),
        },
    )
    .await?;
    let mut finished = false;
    loop {
        match receive(&mut session.stream).await?.body {
            Some(server_frame::Body::Accepted(_)) => return Ok(()),
            Some(server_frame::Body::Credit(grant)) => {
                session.credit = session
                    .credit
                    .saturating_add(grant.additional_bytes as usize);
            }
            Some(server_frame::Body::Checkpoint(_)) if !finished => {
                send(
                    &mut session.sink,
                    &ClientFrame {
                        operation_id: operation.clone(),
                        body: Some(client_frame::Body::FinishUpload(FinishUpload {
                            manifest_digest: plan.manifest_digest.clone(),
                        })),
                    },
                )
                .await?;
                finished = true;
            }
            Some(server_frame::Body::Rejected(failed) | server_frame::Body::Failed(failed)) => {
                return Err(outcome_error(&failed));
            }
            Some(_) => {}
            None => return Err(AppError::shared("empty synchronization response")),
        }
    }
}

async fn finish_backfill(
    session: &mut PublisherSession,
    backfill_id: &str,
    plan: &BackfillPlan,
) -> Result<(), AppError> {
    send(
        &mut session.sink,
        &ClientFrame {
            operation_id: format!("finish-backfill:{backfill_id}"),
            body: Some(client_frame::Body::FinishBackfill(FinishBackfill {
                backfill_id: backfill_id.to_owned(),
                baseline_revision: plan.baseline_revision,
                manifest_digest: plan.manifest_digest.clone(),
            })),
        },
    )
    .await?;
    loop {
        match receive(&mut session.stream).await?.body {
            Some(server_frame::Body::BackfillComplete(_)) => return Ok(()),
            Some(server_frame::Body::Rejected(failed) | server_frame::Body::Failed(failed)) => {
                return Err(outcome_error(&failed));
            }
            Some(_) => {}
            None => return Err(AppError::shared("empty synchronization response")),
        }
    }
}

fn views_fingerprint(start: &protocol::Start) -> String {
    start.views_build.as_ref().map_or_else(
        || "unspecified".to_owned(),
        |build| serde_json::to_string(build).unwrap_or_default(),
    )
}

/// Wait for the first `Accepted`, honouring intervening credit/checkpoint frames.
async fn expect_accepted(
    stream: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    credit: &mut usize,
) -> Result<(), AppError> {
    loop {
        match receive(stream).await?.body {
            Some(server_frame::Body::Accepted(accepted)) => {
                *credit = credit.saturating_add(accepted.credit_bytes as usize);
                return Ok(());
            }
            Some(server_frame::Body::Credit(grant)) => {
                *credit = credit.saturating_add(grant.additional_bytes as usize);
            }
            Some(server_frame::Body::Rejected(failed) | server_frame::Body::Failed(failed)) => {
                return Err(outcome_error(&failed));
            }
            Some(_) => {}
            None => return Err(AppError::shared("empty synchronization response")),
        }
    }
}

fn indexed(revision: i64, views_current: bool) -> protocol::Indexed {
    protocol::Indexed {
        counts: crate::storage::Counts::default(),
        revision,
        removed_messages: 0,
        purged_chats: 0,
        views_current,
    }
}

#[cfg(test)]
/// Close every cached publisher session. Tests use this once a publication is
/// done so a fake server that answers on disconnect can finish.
pub fn close_sessions() {
    let mut slots = session_slots()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    slots.clear();
}

#[cfg(test)]
mod tests;

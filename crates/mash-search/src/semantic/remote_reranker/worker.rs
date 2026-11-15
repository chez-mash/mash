//! A reranking worker for this machine. It answers the frames the remote
//! client sends, with Core ML on Apple Silicon and the CPU model elsewhere,
//! so one long-lived process pays for model compilation instead of every search.
use super::{
    MAX_DOCUMENT_BYTES, MAX_DOCUMENTS, MAX_QUERY_BYTES, MAX_SCORE_MAGNITUDE, RECIPE,
    REQUEST_ID_BYTES, REQUEST_MAGIC, RESPONSE_MAGIC,
};
use crate::AppError;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use std::io::Read;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub const TOKEN_ENV: &str = "MASH_RERANKING_TOKEN";
// Shared with deployment/reranking/contract.py.
const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;
const BATCH: usize = 4;

#[cfg(test)]
mod tests;

/// One raw logit per document, in input order.
pub(super) type Scorer = Arc<dyn Fn(&str, &[&str]) -> Result<Vec<f32>, AppError> + Send + Sync>;

pub struct Engine {
    cpu: fastembed::TextRerank,
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    accelerated: Option<crate::semantic::coreml::reranker::Pool>,
}

impl Engine {
    pub fn load(root: &Path) -> Result<Self, AppError> {
        let cpu = crate::semantic::models::load_reranker(root)?;
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let accelerated = Some(crate::semantic::coreml::reranker::Pool::new(
            root,
            cpu.tokenizer.clone(),
        ));
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let accelerated = accelerated.and_then(|mut pool| {
            // Compile and load every input shape now. A search waits at most
            // its short time bound, and loading a Core ML shape takes seconds.
            match pool.warm() {
                Ok(()) => Some(pool),
                Err(error) => {
                    tracing::warn!(reason = %error.error.message, "Core ML reranking is unavailable; this worker uses CPU");
                    None
                }
            }
        });
        let mut engine = Self {
            cpu,
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            accelerated,
        };
        engine.scores("warm up", &["warm up"])?;
        Ok(engine)
    }

    /// Whether scores come from Core ML and not the CPU model.
    pub const fn accelerated(&self) -> bool {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        if self.accelerated.is_some() {
            return true;
        }
        false
    }

    pub const fn name(&self) -> &'static str {
        if self.accelerated() { "core-ml" } else { "cpu" }
    }

    pub fn scores(&mut self, query: &str, documents: &[&str]) -> Result<Vec<f32>, AppError> {
        let mut order = (0..documents.len()).collect::<Vec<_>>();
        // Similar lengths share a batch, so short candidates are not padded to a long one.
        order.sort_by_key(|&index| documents[index].len());
        let mut scores = vec![f32::NEG_INFINITY; documents.len()];
        for indices in order.chunks(BATCH) {
            let batch = indices
                .iter()
                .map(|&index| documents[index])
                .collect::<Vec<_>>();
            for (&index, score) in indices.iter().zip(self.batch(query, &batch)?) {
                scores[index] = score;
            }
        }
        Ok(scores)
    }

    fn batch(&mut self, query: &str, documents: &[&str]) -> Result<Vec<f32>, AppError> {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        if let Some(pool) = &mut self.accelerated {
            match pool.run(query, documents) {
                Ok(scores) => return Ok(scores),
                Err(error) => {
                    tracing::warn!(reason = %error.error.message, "Core ML reranking failed; this worker now uses CPU");
                    // Do not retry compilation or a failing device on every batch.
                    self.accelerated = None;
                }
            }
        }
        let results = self
            .cpu
            .rerank(query, documents, false, Some(BATCH))
            .map_err(|error| AppError::model(format!("reranking inference failed: {error}")))?;
        let mut scores = vec![None; documents.len()];
        for result in results {
            *scores
                .get_mut(result.index)
                .ok_or_else(|| AppError::model("reranker returned an invalid document index"))? =
                Some(result.score);
        }
        scores
            .into_iter()
            .collect::<Option<_>>()
            .ok_or_else(|| AppError::model("reranker returned too few scores"))
    }
}

/// Serve `POST /rerank` until the process ends. Plain HTTP, so loopback only;
/// the client refuses plain HTTP anywhere else.
pub async fn serve(listen: SocketAddr, models: PathBuf) -> Result<(), AppError> {
    if !listen.ip().is_loopback() {
        return Err(AppError::usage(
            "the reranking worker listens on this machine only, for example 127.0.0.1:8000; put HTTPS in front of it to reach it from another machine",
        ));
    }
    let token = std::env::var(TOKEN_ENV)
        .ok()
        .filter(|token| !token.trim().is_empty())
        .ok_or_else(|| {
            AppError::configuration(format!(
                "set {TOKEN_ENV} to the token searches will present to this worker"
            ))
        })?;
    let engine = crate::blocking(move || Engine::load(&models)).await?;
    let name = engine.name();
    let engine = Mutex::new(engine);
    let scorer: Scorer = Arc::new(move |query, documents| {
        engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .scores(query, documents)
    });
    let listener = tokio::net::TcpListener::bind(listen)
        .await
        .map_err(AppError::io)?;
    eprintln!("Mash reranking worker ready at http://{listen}/rerank ({name})");
    axum::serve(listener, router(&token, scorer, Arc::default()))
        .await
        .map_err(AppError::io)
}

#[derive(Clone)]
struct Worker {
    authorization: blake3::Hash,
    scorer: Scorer,
    /// Authorized, well-formed requests so far, warm-ups included.
    requests: Arc<AtomicU64>,
}

pub(super) fn router(token: &str, scorer: Scorer, requests: Arc<AtomicU64>) -> axum::Router {
    axum::Router::new()
        .route("/rerank", post(rerank))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(Worker {
            authorization: blake3::hash(format!("Bearer {token}").as_bytes()),
            scorer,
            requests,
        })
}

async fn rerank(State(worker): State<Worker>, headers: HeaderMap, body: Bytes) -> Response {
    let supplied = headers
        .get(header::AUTHORIZATION)
        .map_or(&[][..], |value| value.as_bytes());
    // Comparing digests takes the same time wherever the tokens differ.
    if blake3::hash(supplied) != worker.authorization {
        return refuse(StatusCode::UNAUTHORIZED, "invalid reranking token");
    }
    let frame = match headers.get(header::CONTENT_ENCODING) {
        None => body.to_vec(),
        Some(encoding) if encoding == "zstd" => match decompress(&body) {
            Some(frame) => frame,
            None => return refuse(StatusCode::BAD_REQUEST, "invalid reranking batch"),
        },
        Some(_) => {
            return refuse(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "unsupported content encoding",
            );
        }
    };
    let batch = match decode(&frame) {
        Ok(batch) => batch,
        // Refusal messages never contain input text.
        Err(refusal) => return refuse(StatusCode::BAD_REQUEST, refusal.message()),
    };
    worker.requests.fetch_add(1, Ordering::Relaxed);
    if batch.documents.is_empty() {
        // Warm-up: the process and model are loaded; no inference.
        return reply(&batch.request_id, 0, &[]);
    }
    let score = Arc::clone(&worker.scorer);
    let request_id = batch.request_id;
    let outcome = tokio::task::spawn_blocking(move || {
        let started = std::time::Instant::now();
        let documents = batch
            .documents
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let logits = score(&batch.query, &documents)?;
        let elapsed = u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX);
        Ok::<_, AppError>((elapsed, logits))
    })
    .await;
    match outcome {
        Ok(Ok((elapsed, logits))) => reply(&request_id, elapsed, &logits),
        _ => refuse(StatusCode::INTERNAL_SERVER_ERROR, "reranking failed"),
    }
}

fn refuse(status: StatusCode, message: &'static str) -> Response {
    (status, message).into_response()
}

fn reply(request_id: &[u8; REQUEST_ID_BYTES], milliseconds: u32, scores: &[f32]) -> Response {
    match encode_reply(request_id, milliseconds, scores) {
        Some(bytes) => {
            ([(header::CONTENT_TYPE, "application/octet-stream")], bytes).into_response()
        }
        None => refuse(StatusCode::INTERNAL_SERVER_ERROR, "invalid model output"),
    }
}

fn decompress(body: &[u8]) -> Option<Vec<u8>> {
    let mut frame = Vec::new();
    zstd::stream::read::Decoder::new(body)
        .ok()?
        .take(MAX_BODY_BYTES as u64 + 1)
        .read_to_end(&mut frame)
        .ok()?;
    (frame.len() <= MAX_BODY_BYTES).then_some(frame)
}

#[derive(Debug, PartialEq, Eq)]
enum Refusal {
    Malformed,
    Recipe,
    QueryLimit,
    DocumentLimit,
}

impl Refusal {
    const fn message(&self) -> &'static str {
        match self {
            Self::Malformed => "invalid reranking batch",
            Self::Recipe => "unsupported reranking recipe",
            Self::QueryLimit => "query byte limit exceeded",
            Self::DocumentLimit => "document byte limit exceeded",
        }
    }
}

#[derive(Debug)]
struct Batch {
    request_id: [u8; REQUEST_ID_BYTES],
    query: String,
    /// Indexed by document ID. None means a warm-up request.
    documents: Vec<String>,
}

/// Takes the next `length` bytes of a frame, or refuses a truncated one.
fn take<'a>(rest: &mut &'a [u8], length: usize) -> Result<&'a [u8], Refusal> {
    let (taken, after) = rest.split_at_checked(length).ok_or(Refusal::Malformed)?;
    *rest = after;
    Ok(taken)
}

fn word(rest: &mut &[u8]) -> Result<usize, Refusal> {
    let bytes = take(rest, 4)?.try_into().map_err(|_| Refusal::Malformed)?;
    usize::try_from(u32::from_le_bytes(bytes)).map_err(|_| Refusal::Malformed)
}

fn text(rest: &mut &[u8], length: usize) -> Result<String, Refusal> {
    std::str::from_utf8(take(rest, length)?)
        .map(str::to_owned)
        .map_err(|_| Refusal::Malformed)
}

/// Parse the bounded request frame, refusing every malformed shape.
fn decode(frame: &[u8]) -> Result<Batch, Refusal> {
    let mut rest = frame;
    if take(&mut rest, REQUEST_MAGIC.len())? != REQUEST_MAGIC {
        return Err(Refusal::Malformed);
    }
    let recipe_length = take(&mut rest, 2)?
        .try_into()
        .map(u16::from_le_bytes)
        .map_err(|_| Refusal::Malformed)?;
    if take(&mut rest, usize::from(recipe_length))? != RECIPE.as_bytes() {
        return Err(Refusal::Recipe);
    }
    let request_id = take(&mut rest, REQUEST_ID_BYTES)?
        .try_into()
        .map_err(|_| Refusal::Malformed)?;
    let query_length = word(&mut rest)?;
    if query_length > MAX_QUERY_BYTES {
        return Err(Refusal::QueryLimit);
    }
    let query = text(&mut rest, query_length)?;
    let count = word(&mut rest)?;
    if count > MAX_DOCUMENTS || (count == 0) != query.is_empty() {
        return Err(Refusal::Malformed);
    }
    let mut documents = vec![None; count];
    let mut total = 0usize;
    for _ in 0..count {
        let id = word(&mut rest)?;
        let length = word(&mut rest)?;
        total = total.saturating_add(length);
        let slot = documents.get_mut(id).ok_or(Refusal::Malformed)?;
        if slot.is_some() {
            return Err(Refusal::Malformed);
        }
        *slot = Some(text(&mut rest, length)?);
    }
    if !rest.is_empty() {
        return Err(Refusal::Malformed);
    }
    if total > MAX_DOCUMENT_BYTES {
        return Err(Refusal::DocumentLimit);
    }
    Ok(Batch {
        request_id,
        query,
        documents: documents
            .into_iter()
            .collect::<Option<_>>()
            .ok_or(Refusal::Malformed)?,
    })
}

/// One little-endian f32 logit per document ID, or nothing for a score the
/// client would refuse.
fn encode_reply(
    request_id: &[u8; REQUEST_ID_BYTES],
    milliseconds: u32,
    scores: &[f32],
) -> Option<Vec<u8>> {
    let count = u32::try_from(scores.len()).ok()?;
    let mut bytes = Vec::with_capacity(4 + REQUEST_ID_BYTES + 8 + scores.len() * 8);
    bytes.extend_from_slice(RESPONSE_MAGIC);
    bytes.extend_from_slice(request_id);
    bytes.extend_from_slice(&milliseconds.to_le_bytes());
    bytes.extend_from_slice(&count.to_le_bytes());
    for (id, score) in (0..count).zip(scores) {
        if !score.is_finite() || score.abs() > MAX_SCORE_MAGNITUDE {
            return None;
        }
        bytes.extend_from_slice(&id.to_le_bytes());
        bytes.extend_from_slice(&score.to_le_bytes());
    }
    Some(bytes)
}

/// The worker contract as an `OpenAPI` document. Every worker (this one and the
/// Python ones in `deployment/reranking`) and the client answer to it; the
/// committed copy is `deployment/reranking/openapi.json`.
pub fn document() -> serde_json::Value {
    let refusal = |description: &str| {
        serde_json::json!({
            "description": description,
            "content": { "text/plain": { "schema": { "type": "string" } } },
        })
    };
    serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": "mash-reranking-worker",
            "version": "1",
            "description": "Scores one search query against its candidate passages. A worker runs wherever a suitable accelerator is; Mash reaches it with `[reranking] backend = \"remote\"`. Bodies are versioned binary frames with little-endian integers. Generated from crates/mash-search/src/semantic/remote_reranker/worker.rs; do not edit.",
        },
        "x-mash-contract": {
            "recipe": RECIPE,
            "request_magic": std::str::from_utf8(REQUEST_MAGIC).unwrap_or_default(),
            "response_magic": std::str::from_utf8(RESPONSE_MAGIC).unwrap_or_default(),
            "request_id_bytes": REQUEST_ID_BYTES,
            "max_query_bytes": MAX_QUERY_BYTES,
            "max_documents": MAX_DOCUMENTS,
            "max_document_bytes": MAX_DOCUMENT_BYTES,
            "max_body_bytes": MAX_BODY_BYTES,
            "max_score_magnitude": MAX_SCORE_MAGNITUDE,
        },
        "paths": {
            "/rerank": {
                "post": {
                    "operationId": "rerank",
                    "summary": "Score a query against its documents, or warm the worker up",
                    "description": "One attempt per search pass: the client never retries, and reranks on its own CPU when the worker does not answer within its time bound. A frame with an empty query and no documents is a warm-up and runs no inference. The worker refuses a recipe other than its own, so its scores are interchangeable with the client's CPU scores.",
                    "security": [{ "token": [] }],
                    "parameters": [{
                        "name": "Content-Encoding",
                        "in": "header",
                        "required": false,
                        "description": "Present when the body is a zstd-compressed frame. Clients compress frames of one kilobyte or more. The decompressed frame has the same byte limit as a plain one.",
                        "schema": { "type": "string", "enum": ["zstd"] },
                    }],
                    "requestBody": {
                        "required": true,
                        "description": "Request frame, in order: the 4-byte magic `MRK1`; u16 recipe length and the recipe as UTF-8; a 32-byte request ID chosen by the client; u32 query length and the query as UTF-8; u32 document count; then for each document a u32 ID, a u32 length and the text as UTF-8. Document IDs are unique and cover 0 to count-1. Nothing follows the last document.",
                        "content": { "application/octet-stream": {
                            "schema": { "type": "string", "format": "binary", "maxLength": MAX_BODY_BYTES },
                        } },
                    },
                    "responses": {
                        "200": {
                            "description": "Reply frame, in order: the 4-byte magic `MRR1`; the request's 32-byte ID; u32 inference time in milliseconds; u32 score count; then for each document a u32 ID and an f32 raw logit. There is one finite score per document, within the stated magnitude. A warm-up reply has no scores.",
                            "content": { "application/octet-stream": {
                                "schema": { "type": "string", "format": "binary" },
                            } },
                        },
                        "400": refusal("The frame is malformed, exceeds a query or document byte limit, or names another recipe. The message says which and never contains input text."),
                        "401": refusal("The bearer token is missing or wrong."),
                        "413": refusal("The body exceeds the byte limit."),
                        "415": refusal("The content encoding is not zstd."),
                        "500": refusal("The model failed or produced a score outside the contract."),
                    },
                },
            },
        },
        "components": {
            "securitySchemes": {
                "token": {
                    "type": "http",
                    "scheme": "bearer",
                    "description": "The value of the environment variable the node's `token_env` names.",
                },
            },
        },
    })
}

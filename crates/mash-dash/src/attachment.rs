//! One attachment's bytes for the transcript, read through the same core as
//! the MCP `get_attachment` tool and from the same source as the chat page
//! that lists it, so its retention, integrity and redaction rules apply here
//! unchanged. Archived files are untrusted: only raster images a browser
//! cannot run as a page are shown inline, everything else is a download, and
//! no answer may be sniffed into another type.
use std::fmt::Write as _;
use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::{Engine, engine::general_purpose::STANDARD};
use mash_app::core::{self, Operation, Request};
use mash_model::AppError;

use super::Dash;

pub(crate) const PATH: &str = "/api/attachment";

/// Images every browser draws without running anything in them. SVG is left
/// out on purpose: it can carry script.
const INLINE: &[&str] = &["image/png", "image/jpeg", "image/gif", "image/webp"];

/// An answer for the digest the page asked for: those bytes never change.
const KEPT: &str = "private, max-age=31536000, immutable";
/// An answer without one is checked again against its digest each time.
const CHECKED: &str = "private, no-cache";

pub(crate) fn routes(router: ApiRouter<Arc<Dash>>) -> ApiRouter<Arc<Dash>> {
    use aide::axum::routing::get_with;
    router.api_route(
        PATH,
        get_with(attachment, |operation| {
            super::contract::reads::<String>(
                PATH,
                "One retained attachment of a message, as its bytes.",
            )(operation)
        }),
    )
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct AttachmentParams {
    /// The message's ID.
    message: String,
    /// The attachment's ID, as the chat page lists it.
    attachment: String,
    /// The node the chat page was read with.
    node: Option<String>,
    /// Whether the chat page was read from the shared index.
    #[serde(default)]
    shared: bool,
    /// The digest the page was shown. When it still matches, the answer may
    /// be kept; when it does not, the attachment changed and is refused.
    digest: Option<String>,
}

async fn attachment(
    State(state): State<Arc<Dash>>,
    headers: HeaderMap,
    Query(params): Query<AttachmentParams>,
) -> Response {
    let request = Request {
        paths: state.paths.clone(),
        operation: Operation::ChatAttachment(core::ChatAttachment {
            message_id: params.message,
            attachment_id: params.attachment,
            node: params.node,
            shared: params.shared,
        }),
    };
    let file = match core::execute(request).await {
        Ok(core::Response::GetAttachment(file)) => file,
        Ok(_) => {
            return super::json_result(Err(AppError::internal("unexpected attachment answer")));
        }
        Err(error) => return super::json_result(Err(error)),
    };
    let digest = file.attachment.digest.clone().unwrap_or_default();
    if params.digest.as_ref().is_some_and(|asked| *asked != digest) {
        return super::json_result(Err(AppError::not_found(
            "this attachment has changed since the chat was read; reload the chat",
        )));
    }
    let cache = if params.digest.is_some() {
        KEPT
    } else {
        CHECKED
    };
    let tag = format!("\"{digest}\"");
    let mut answer = served(&file.attachment.media_type, Some(&file.attachment.name));
    answer.push((header::CACHE_CONTROL, HeaderValue::from_static(cache)));
    if let Ok(value) = HeaderValue::from_str(&tag) {
        answer.push((header::ETAG, value));
    }
    let unchanged = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|candidate| candidate.trim() == tag));
    if unchanged {
        return with_headers(StatusCode::NOT_MODIFIED, answer, Vec::new());
    }
    match STANDARD.decode(file.data) {
        Ok(bytes) => with_headers(StatusCode::OK, answer, bytes),
        Err(_) => super::json_result(Err(AppError::database_data(
            "invalid retained attachment bytes",
        ))),
    }
}

/// Another machine's answer for one of its attachments, passed on with
/// headers this dashboard chose: the type decides inline or download again,
/// and only the cache and digest values this module itself sends are kept.
pub(crate) fn relay(from: &reqwest::header::HeaderMap, body: Vec<u8>) -> Response {
    let text = |name: HeaderName| from.get(name).and_then(|value| value.to_str().ok());
    let mut answer = served(text(header::CONTENT_TYPE).unwrap_or_default(), None);
    let cache = if text(header::CACHE_CONTROL) == Some(KEPT) {
        KEPT
    } else {
        CHECKED
    };
    answer.push((header::CACHE_CONTROL, HeaderValue::from_static(cache)));
    if let Some(tag) = text(header::ETAG).filter(|tag| digest_tag(tag))
        && let Ok(value) = HeaderValue::from_str(tag)
    {
        answer.push((header::ETAG, value));
    }
    with_headers(StatusCode::OK, answer, body)
}

fn digest_tag(tag: &str) -> bool {
    tag.strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .is_some_and(|hex| {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
}

/// Headers an attachment answer chose, which the dashboard's own hardening
/// applies over its defaults: a cache by digest and a sandboxing policy in
/// place of the page's.
#[derive(Clone)]
pub(crate) struct Chosen(pub(crate) Vec<(HeaderName, HeaderValue)>);

fn with_headers(
    status: StatusCode,
    headers: Vec<(HeaderName, HeaderValue)>,
    body: Vec<u8>,
) -> Response {
    let mut response = (status, body).into_response();
    for (name, value) in &headers {
        response.headers_mut().insert(name, value.clone());
    }
    response.extensions_mut().insert(Chosen(headers));
    response
}

/// The type, disposition and safety headers for a file of `media_type`.
/// A file that is not shown inline is a download named for `name`, and is
/// sandboxed should a browser open it anyway.
fn served(media_type: &str, name: Option<&str>) -> Vec<(HeaderName, HeaderValue)> {
    let media_type = media_type.trim().to_ascii_lowercase();
    let inline = INLINE.contains(&media_type.as_str());
    let kind = if inline || plain_type(&media_type) {
        media_type.as_str()
    } else {
        "application/octet-stream"
    };
    let disposition = if inline {
        "inline".to_owned()
    } else {
        name.map_or_else(|| "attachment".to_owned(), download_name)
    };
    let mut headers = vec![
        (
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ),
        (
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'none'; sandbox"),
        ),
    ];
    if let Ok(value) = HeaderValue::from_str(kind) {
        headers.push((header::CONTENT_TYPE, value));
    }
    if let Ok(value) = HeaderValue::from_str(&disposition) {
        headers.push((header::CONTENT_DISPOSITION, value));
    }
    headers
}

/// A bare `type/subtype` of token characters, without parameters.
fn plain_type(media_type: &str) -> bool {
    let token = |part: &str| {
        !part.is_empty()
            && part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"!#$&-^_.+".contains(&b))
    };
    media_type
        .split_once('/')
        .is_some_and(|(kind, subtype)| token(kind) && token(subtype))
}

/// `attachment` with an ASCII `filename` and the exact UTF-8 `filename*`.
fn download_name(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        return "attachment".to_owned();
    }
    let ascii: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || " ._-()+".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    let mut encoded = String::new();
    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() || b"._-".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    format!("attachment; filename=\"{ascii}\"; filename*=UTF-8''{encoded}")
}

#[cfg(test)]
mod tests;

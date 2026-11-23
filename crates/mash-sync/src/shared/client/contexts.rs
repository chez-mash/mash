//! Bounded typed NDJSON context reads retain evidence received before timeout.
use super::{ADMISSION, Endpoint, protocol, request};
use crate::{AppError, config::SharedConfig, storage::Message};
use futures_util::StreamExt;
use std::collections::BTreeSet;
use tracing::Instrument;

type Completed = Vec<((String, String), Result<Vec<Message>, AppError>)>;
const MAX_LINE: usize = 4 * 1024 * 1024;
const MAX_BODY: usize = 64 * 1024 * 1024;

pub(in crate::shared) async fn read(
    config: &SharedConfig,
    anchors: Vec<(String, String)>,
) -> (Completed, Option<String>) {
    if anchors.is_empty() {
        return (Vec::new(), None);
    }
    let span = tracing::info_span!(
        "shared_http_request",
        endpoint = "contexts",
        trace_id = tracing::field::Empty,
        status = tracing::field::Empty,
        elapsed_ms = tracing::field::Empty,
        outcome = tracing::field::Empty,
    );
    let mut completed = Vec::new();
    let started = std::time::Instant::now();
    let result = tokio::time::timeout(
        super::super::retrieval::CONTEXT_DEADLINE,
        async {
            let _permit = Endpoint::Contexts
                .capacity(&ADMISSION)
                .acquire()
                .await
                .map_err(|_| invalid())?;
            let mut request = request(config, reqwest::Method::POST, Endpoint::Contexts)?;
            if !span.is_disabled()
                && let Some(id) = crate::observability::http::TraceId::generate()
            {
                span.record("trace_id", id.as_str());
                request = request.header(crate::observability::http::HEADER, id.as_str());
            }
            let input = protocol::Contexts {
                anchors: anchors
                    .iter()
                    .map(|(node, id)| protocol::View {
                        node_id: node.clone(),
                        id: id.clone(),
                        context: 2,
                        content: crate::storage::MessageContent::SearchProjection,
                    })
                    .collect(),
            };
            let response = request.json(&input).send().await.map_err(|_| {
                AppError::shared("shared context HTTP request failed; check API and network")
            })?;
            span.record("status", response.status().as_u16());
            if !response.status().is_success() {
                return Err(AppError::shared("shared context request was rejected"));
            }
            let mut seen = BTreeSet::new();
            let mut lines = std::pin::pin!(frames(response.bytes_stream()));
            while let Some(line) = lines.next().await {
                let line = line?;
                // A frame can be megabytes of JSON; parse it off the runtime.
                let item = crate::blocking(move || parse(&line)).await?;
                let index = item.index();
                let key = anchors.get(index).ok_or_else(invalid)?;
                if !seen.insert(index) {
                    return Err(invalid());
                }
                let result = match item {
                    protocol::ContextResult::Complete { messages, .. } => Ok(messages),
                    protocol::ContextResult::Failed { error, .. } => Err(error),
                };
                completed.push((key.clone(), result));
            }
            if seen.len() != anchors.len() {
                return Err(invalid());
            }
            Ok(())
        }
        .instrument(span.clone()),
    )
    .await;
    let error = match result {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(error.error.message),
        Err(_) => Some(
            "shared context deadline exceeded; ranking uses available message evidence".to_owned(),
        ),
    };
    span.record("elapsed_ms", crate::progress::milliseconds(started));
    span.record(
        "outcome",
        if error.is_none() {
            "success"
        } else {
            "partial"
        },
    );
    (completed, error)
}

fn invalid() -> AppError {
    AppError::shared("invalid shared context HTTP response")
}

/// The body's NDJSON frames, through the shared line decoder. The whole body
/// is capped at `MAX_BODY` and each frame at `MAX_LINE`; a body cut off partway
/// through a frame is invalid, since every frame must arrive complete.
fn frames<S, B, E>(bytes: S) -> impl futures_util::Stream<Item = Result<Vec<u8>, AppError>>
where
    S: futures_util::Stream<Item = Result<B, E>>,
    B: AsRef<[u8]>,
{
    capped_frames(bytes, MAX_BODY)
}

fn capped_frames<S, B, E>(
    bytes: S,
    max_body: usize,
) -> impl futures_util::Stream<Item = Result<Vec<u8>, AppError>>
where
    S: futures_util::Stream<Item = Result<B, E>>,
    B: AsRef<[u8]>,
{
    let mut total = 0_usize;
    let counted = bytes.map(move |chunk| {
        let chunk = chunk.map_err(|_| invalid())?;
        total = total.saturating_add(chunk.as_ref().len());
        if total > max_body {
            return Err(invalid());
        }
        Ok(chunk)
    });
    mash_model::lines::lines(counted, MAX_LINE).map(|line| {
        line.map_err(|error| match error {
            mash_model::lines::LineError::Source(error) => error,
            mash_model::lines::LineError::TooLong
            | mash_model::lines::LineError::Unterminated(_) => invalid(),
        })
    })
}

fn parse(line: &[u8]) -> Result<protocol::ContextResult, AppError> {
    serde_json::from_slice(line).map_err(|_| invalid())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn decode(chunks: Vec<&[u8]>) -> Result<Vec<protocol::ContextResult>, AppError> {
        let source = futures_util::stream::iter(chunks.into_iter().map(Ok::<_, AppError>));
        let lines: Vec<Result<Vec<u8>, AppError>> = frames(source).collect().await;
        lines.into_iter().map(|line| parse(&line?)).collect()
    }

    // Split points themselves are the shared decoder's tests; this checks the
    // reader's own framing on top of it.
    #[tokio::test]
    async fn frames_parse_in_any_index_order_across_chunks() {
        let bytes = b"{\"status\":\"complete\",\"index\":1,\"messages\":[]}\n{\"status\":\"complete\",\"index\":0,\"messages\":[]}\n";
        let (first, second) = bytes.split_at(30);
        let items = decode(vec![first, second]).await.unwrap();
        assert_eq!(
            items
                .iter()
                .map(protocol::ContextResult::index)
                .collect::<Vec<_>>(),
            [1, 0]
        );
    }

    #[tokio::test]
    async fn malformed_unknown_cut_off_and_oversized_frames_fail_closed() {
        for bytes in [
            b"{}\n".as_slice(),
            b"{\"status\":\"complete\",\"index\":0,\"messages\":[],\"extra\":true}\n",
            b"\n",
            // Cut off before its newline.
            b"{\"status\":\"complete\",\"index\":0,\"messages\":[]}",
        ] {
            assert!(
                decode(vec![bytes]).await.is_err(),
                "{:?}",
                String::from_utf8_lossy(bytes)
            );
        }
        assert!(decode(vec![&vec![b'x'; MAX_LINE + 1]]).await.is_err());
        // The whole body is capped too, across chunks.
        let frame = b"{\"status\":\"complete\",\"index\":0,\"messages\":[]}\n";
        let source = futures_util::stream::iter([Ok::<_, AppError>(&frame[..]), Ok(&frame[..])]);
        let lines: Vec<_> = capped_frames(source, frame.len() + 1).collect().await;
        assert!(lines[0].is_ok());
        assert!(lines[1].is_err(), "the second frame passes the body cap");
    }
}

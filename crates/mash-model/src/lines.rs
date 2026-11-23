//! One decoder for byte streams that carry one record per line: server-sent
//! events, NDJSON. Every reader of such a stream used to split lines by hand,
//! each with its own size limit; this is the one place that does it.
//!
//! A line is yielded without its `\n`. A `\r` before it is kept, so a reader
//! that accepts `\r\n` trims it as it already trims the rest of the line. The
//! decoder holds at most `max_line` bytes of one unfinished line; a longer line
//! ends the stream with [`LineError::TooLong`] rather than growing without
//! bound. Bytes after the last `\n` when the body ends are reported as
//! [`LineError::Unterminated`], so a reader that needs every line complete can
//! refuse a cut-off body, and one that does not can ignore it.
use futures_util::{Stream, StreamExt};

/// Why a line stream ended early.
#[derive(Debug, PartialEq, Eq)]
pub enum LineError<E> {
    /// The byte stream itself failed.
    Source(E),
    /// One line grew past the limit before its `\n` arrived.
    TooLong,
    /// The body ended partway through a line; these are its bytes.
    Unterminated(Vec<u8>),
}

/// The lines of `bytes`, each at most `max_line` bytes long. The stream ends
/// after its first error.
pub fn lines<S, B, E>(
    bytes: S,
    max_line: usize,
) -> impl Stream<Item = Result<Vec<u8>, LineError<E>>>
where
    S: Stream<Item = Result<B, E>>,
    B: AsRef<[u8]>,
{
    async_stream::stream! {
        let mut bytes = std::pin::pin!(bytes);
        let mut pending: Vec<u8> = Vec::new();
        while let Some(chunk) = bytes.next().await {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(error) => {
                    yield Err(LineError::Source(error));
                    return;
                }
            };
            for piece in chunk.as_ref().split_inclusive(|byte| *byte == b'\n') {
                let (body, ended) = match piece.split_last() {
                    Some((b'\n', body)) => (body, true),
                    _ => (piece, false),
                };
                if pending.len().saturating_add(body.len()) > max_line {
                    yield Err(LineError::TooLong);
                    return;
                }
                pending.extend_from_slice(body);
                if ended {
                    yield Ok(std::mem::take(&mut pending));
                }
            }
        }
        if !pending.is_empty() {
            yield Err(LineError::Unterminated(pending));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LineError, lines};
    use futures_util::StreamExt;

    type Lines = Vec<Result<Vec<u8>, LineError<&'static str>>>;

    async fn split(chunks: Vec<&[u8]>, max_line: usize) -> Lines {
        let source = futures_util::stream::iter(chunks.into_iter().map(Ok::<_, &'static str>));
        lines(source, max_line).collect().await
    }

    #[tokio::test]
    async fn every_split_point_gives_the_same_lines() {
        let bytes = b"data: one\n\ndata: two\r\n{\"x\":1}\n";
        let expected: Lines = vec![
            Ok(b"data: one".to_vec()),
            Ok(Vec::new()),
            Ok(b"data: two\r".to_vec()),
            Ok(b"{\"x\":1}".to_vec()),
        ];
        for first in 0..=bytes.len() {
            for second in first..=bytes.len() {
                let chunks = vec![&bytes[..first], &bytes[first..second], &bytes[second..]];
                assert_eq!(
                    split(chunks, 64).await,
                    expected,
                    "split at {first}/{second}"
                );
            }
        }
    }

    #[tokio::test]
    async fn a_line_past_the_limit_ends_the_stream() {
        assert_eq!(
            split(vec![b"ok\n", b"12345", b"6\nnever\n"], 5).await,
            vec![Ok(b"ok".to_vec()), Err(LineError::TooLong)]
        );
        // Exactly the limit is still a line.
        assert_eq!(
            split(vec![b"12345\n"], 5).await,
            vec![Ok(b"12345".to_vec())]
        );
    }

    #[tokio::test]
    async fn a_last_line_without_its_newline_is_reported() {
        assert_eq!(
            split(vec![b"done\nhal", b"f"], 64).await,
            vec![
                Ok(b"done".to_vec()),
                Err(LineError::Unterminated(b"half".to_vec()))
            ]
        );
        assert_eq!(split(vec![b"done\n"], 64).await, vec![Ok(b"done".to_vec())]);
    }

    #[tokio::test]
    async fn a_failing_source_ends_the_stream_after_the_lines_it_completed() {
        let source = futures_util::stream::iter([Ok(&b"a\nb"[..]), Err("reset"), Ok(&b"c\n"[..])]);
        let got: Lines = lines(source, 64).collect().await;
        assert_eq!(
            got,
            vec![Ok(b"a".to_vec()), Err(LineError::Source("reset"))]
        );
    }
}

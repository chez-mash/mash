//! One question, answered as a stream of events. The conversation runs inside
//! the stream, so it advances only while the browser reads and stops when the
//! response is dropped; how each question ended is logged once.
use super::{Ask, Assistant, Dash, Event, Stop, converse};
use std::sync::Arc;
use tokio::sync::mpsc;

/// One question's events, ending with `Done` or `Failed`. The conversation
/// runs inside this stream: it only advances while the browser reads, and
/// dropping the response (the reader closed the panel or the page) drops it
/// mid-step, model request and lookups included. The permit is held for the
/// stream's life. `converse` writes to a bounded channel that this stream
/// drains in the same task, so nothing outlives the response.
pub(super) fn answer(
    state: Arc<Dash>,
    assistant: Assistant,
    token: String,
    node: String,
    ask: Ask,
    permit: tokio::sync::OwnedSemaphorePermit,
) -> impl futures_util::Stream<Item = Event> {
    async_stream::stream! {
        let _permit = permit;
        let mut logged = Abandoned::new();
        let (events, mut receiver) = mpsc::channel(32);
        let work = converse(&state, &assistant, &token, node, ask, &events);
        let mut work = std::pin::pin!(work);
        let outcome = loop {
            let step = tokio::select! {
                // Events already written go out before the outcome.
                biased;
                Some(event) = receiver.recv() => Ok(event),
                outcome = &mut work => Err(outcome),
            };
            match step {
                Ok(event) => yield event,
                Err(outcome) => break outcome,
            }
        };
        while let Ok(event) = receiver.try_recv() {
            yield event;
        }
        yield logged.finish(outcome);
    }
}

/// Sends an event to the stream that drains it in the same task: the receiver
/// lives as long as the conversation, so a send cannot fail while it runs.
pub(super) async fn say(events: &mpsc::Sender<Event>, event: Event) {
    let _ = events.send(event).await;
}

/// Logs how a question ended; a question dropped before it ended (the reader
/// left) is logged as abandoned when the stream is dropped.
struct Abandoned {
    started: std::time::Instant,
    ended: bool,
}

impl Abandoned {
    fn new() -> Self {
        Self {
            started: std::time::Instant::now(),
            ended: false,
        }
    }

    fn finish(&mut self, outcome: Result<bool, Stop>) -> Event {
        self.ended = true;
        let elapsed_ms = crate::progress::milliseconds(self.started);
        match outcome {
            Ok(partial) => {
                tracing::info!(
                    outcome = "answered",
                    partial,
                    elapsed_ms,
                    "assistant_question"
                );
                Event::Done { partial }
            }
            Err(Stop::Failed(reason, message)) => {
                tracing::warn!(
                    outcome = "failed",
                    ?reason,
                    elapsed_ms,
                    "assistant_question"
                );
                Event::Failed { reason, message }
            }
        }
    }
}

impl Drop for Abandoned {
    fn drop(&mut self) {
        if !self.ended {
            let elapsed_ms = crate::progress::milliseconds(self.started);
            tracing::info!(outcome = "abandoned", elapsed_ms, "assistant_question");
        }
    }
}

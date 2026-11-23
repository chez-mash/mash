//! The one thread that writes the usage log.
//!
//! Callers hand rows to a bounded channel with `try_send`, so recording costs
//! a queue push and never waits on SQLite or on another process holding the
//! log. A full channel means the writer has fallen behind: the row is dropped
//! and counted, and the count is stored with the next batch the writer
//! commits. The writer is a dedicated thread rather than a runtime task
//! because SQLite blocks; it batches what has queued up into one transaction
//! per log.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::time::{Duration, Instant};

use super::Call;

/// Rows that may wait for the writer. A burst larger than this means the
/// writer is stuck (a disk problem, a lock held for long), not merely busy.
const CAPACITY: usize = 1024;
/// The most rows one transaction commits.
const BATCH: usize = 256;

pub(super) enum Message {
    Call(PathBuf, Box<Call>),
    Flush(mpsc::Sender<()>),
}

/// The sending side of the writer's channel, and the count of rows it dropped.
pub(super) struct Queue {
    sender: SyncSender<Message>,
    dropped: AtomicU64,
}

impl Queue {
    /// A queue whose rows are written by a new writer thread, or `None` when
    /// the thread cannot be started (the rows are then dropped and counted).
    fn start(capacity: usize) -> Option<Self> {
        let (sender, receiver) = mpsc::sync_channel(capacity);
        std::thread::Builder::new()
            .name("mash-usage".to_owned())
            .spawn(move || run(&receiver))
            .ok()?;
        Some(Self {
            sender,
            dropped: AtomicU64::new(0),
        })
    }

    /// Queue `call` without waiting; a full or closed channel drops it and
    /// returns `false`.
    pub(super) fn offer(&self, path: PathBuf, call: Call) -> bool {
        match self.sender.try_send(Message::Call(path, Box::new(call))) {
            Ok(()) => true,
            Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }

    /// Wait until everything queued before this call has been written.
    pub(super) fn flush(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        let (done, written) = mpsc::channel();
        let mut message = Message::Flush(done);
        loop {
            match self.sender.try_send(message) {
                Ok(()) => break,
                Err(TrySendError::Full(returned)) if Instant::now() < deadline => {
                    message = returned;
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(_) => return false,
            }
        }
        written
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .is_ok()
    }

    #[cfg(test)]
    pub(super) fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// A queue nothing reads, for proving that a stalled writer drops rows
    /// instead of blocking the caller. The receiver is returned so the
    /// channel stays open.
    #[cfg(test)]
    pub(super) fn stalled(capacity: usize) -> (Self, Receiver<Message>) {
        let (sender, receiver) = mpsc::sync_channel(capacity);
        (
            Self {
                sender,
                dropped: AtomicU64::new(0),
            },
            receiver,
        )
    }
}

fn queue() -> Option<&'static Queue> {
    static QUEUE: OnceLock<Option<Queue>> = OnceLock::new();
    QUEUE.get_or_init(|| Queue::start(CAPACITY)).as_ref()
}

/// Rows dropped since this process started, not yet stored in a log.
static UNREPORTED: AtomicU64 = AtomicU64::new(0);

pub(super) fn submit(path: PathBuf, call: Call) {
    let queued = queue().is_some_and(|queue| queue.offer(path, call));
    if !queued {
        UNREPORTED.fetch_add(1, Ordering::Relaxed);
    }
}

pub(super) fn flush(timeout: Duration) -> bool {
    queue().is_some_and(|queue| queue.flush(timeout))
}

fn run(receiver: &Receiver<Message>) {
    let mut logs: HashMap<PathBuf, rusqlite::Connection> = HashMap::new();
    while let Ok(first) = receiver.recv() {
        let mut pending = vec![first];
        while pending.len() < BATCH {
            match receiver.try_recv() {
                Ok(message) => pending.push(message),
                Err(_) => break,
            }
        }
        let mut calls: HashMap<PathBuf, Vec<Call>> = HashMap::new();
        let mut flushes = Vec::new();
        for message in pending {
            match message {
                Message::Call(path, call) => calls.entry(path).or_default().push(*call),
                Message::Flush(done) => flushes.push(done),
            }
        }
        for (path, rows) in calls {
            let dropped = UNREPORTED.swap(0, Ordering::Relaxed);
            if let Err(error) = write(&mut logs, &path, &rows, dropped) {
                // A log that cannot be written keeps the drop count for the
                // next one and is opened afresh next time.
                UNREPORTED.fetch_add(dropped, Ordering::Relaxed);
                logs.remove(&path);
                tracing::debug!(target: "mash::usage", error = %error.error.message, "usage log write failed");
            }
        }
        for done in flushes {
            let _ = done.send(());
        }
    }
}

fn write(
    logs: &mut HashMap<PathBuf, rusqlite::Connection>,
    path: &PathBuf,
    rows: &[Call],
    dropped: u64,
) -> Result<(), crate::AppError> {
    if !logs.contains_key(path) {
        // The log lives beside an index, so it is written only once Mash's
        // data directory exists. A command that failed before Mash created
        // it, or `mash uninstall` after removing it, must not bring the
        // directory back just to log the call.
        if !path.parent().is_some_and(Path::is_dir) {
            return Ok(());
        }
        let connection = super::store::open(path)?;
        logs.insert(path.clone(), connection);
    }
    match logs.get_mut(path) {
        Some(connection) => super::store::insert(connection, rows, dropped),
        None => Ok(()),
    }
}

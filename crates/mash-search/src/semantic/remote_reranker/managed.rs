//! The reranking worker Mash runs for itself on Apple Silicon. A search or
//! warm-up that finds none starts one in the background and reranks on the
//! CPU meanwhile; the worker leaves again after an idle period. Nothing is
//! installed and no supervisor is involved: a lock file says whether a worker
//! is alive, and a private state file says where and with which token.
use super::Client;
use super::worker::{self, Scorer};
use crate::AppError;
use crate::observability::retrieval::RerankerFallback;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The first process argument of a worker started by Mash, followed by the
/// models directory. Internal: not part of either command line.
pub const ARGUMENT: &str = "__reranking-worker";
const STATE: &str = "reranking-worker.json";
const LOCK: &str = "reranking-worker.lock";
const FAILED: &str = "reranking-worker.failed";
const LOG: &str = "reranking-worker.log";
/// Long enough to cover the pauses inside a working session, short enough
/// that the compiled models are not held once it ends.
const IDLE: Duration = Duration::from_mins(15);
/// A worker that could not use Core ML is not started again for this long:
/// each attempt costs many seconds of compilation.
const RETRY_AFTER_FAILURE: Duration = Duration::from_hours(1);
/// Local inference takes a fraction of this; past it something is wrong and
/// the search reranks on the CPU.
const TIMEOUT: Duration = Duration::from_millis(1500);

#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize)]
struct State {
    url: String,
    token: String,
}

/// Core ML reranking needs Apple Silicon. Elsewhere the worker would only
/// move CPU inference behind a socket.
pub const fn supported() -> bool {
    cfg!(all(target_os = "macos", target_arch = "aarch64"))
}

/// A client for the running worker. With none running this starts one, unless
/// one failed recently, and answers `None`: rerank on the CPU for now.
pub(super) fn connect(root: &Path) -> Option<Result<Arc<Client>, RerankerFallback>> {
    connect_with(root, &start)
}

fn connect_with(
    root: &Path,
    start: &dyn Fn(&Path),
) -> Option<Result<Arc<Client>, RerankerFallback>> {
    if running(root) {
        // No state yet means the worker is still loading its models.
        let state = read_state(root)?;
        return Some(super::cached_client(&state.url, state.token, TIMEOUT));
    }
    if !recently_failed(root) {
        start(root);
    }
    None
}

fn lock_file(root: &Path) -> Option<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.create(true).truncate(false).read(true).write(true);
    crate::platform::private_file(&mut options);
    options.open(root.join(LOCK)).ok()
}

/// A worker holds the lock from before it loads a model until it exits.
fn running(root: &Path) -> bool {
    lock_file(root).is_some_and(|file| matches!(file.try_lock(), Err(fs::TryLockError::WouldBlock)))
}

fn read_state(root: &Path) -> Option<State> {
    serde_json::from_slice(&fs::read(root.join(STATE)).ok()?).ok()
}

fn recently_failed(root: &Path) -> bool {
    fs::metadata(root.join(FAILED))
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .is_some_and(|age| age < RETRY_AFTER_FAILURE)
}

/// Run this executable again as a worker that outlives the calling process.
fn start(root: &Path) {
    let Ok(executable) = std::env::current_exe() else {
        return;
    };
    let log = fs::File::create(root.join(LOG))
        .map_or_else(|_| std::process::Stdio::null(), std::process::Stdio::from);
    let mut command = std::process::Command::new(executable);
    command
        .arg(ARGUMENT)
        .arg(root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(log);
    #[cfg(unix)]
    {
        // Its own process group: an interrupt aimed at the search that
        // started it does not stop the worker.
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    if let Ok(mut child) = command.spawn() {
        // Collect the exit status if this process is still around for it.
        std::thread::spawn(move || {
            let _ = child.wait();
        });
    }
}

/// Runs the worker when the process arguments ask for one. `None` means the
/// arguments are an ordinary command line.
pub fn entry(args: impl IntoIterator<Item = OsString>) -> Option<i32> {
    let mut args = args.into_iter().skip(1);
    if args.next()? != ARGUMENT {
        return None;
    }
    let root = PathBuf::from(args.next()?);
    let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    else {
        return Some(1);
    };
    Some(match runtime.block_on(run(&root)) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{}", error.error.message);
            1
        }
    })
}

async fn run(root: &Path) -> Result<(), AppError> {
    let lock = lock_file(root).ok_or_else(|| AppError::model("cannot open the worker lock"))?;
    if lock.try_lock().is_err() {
        // Another worker got here first.
        return Ok(());
    }
    let _ = fs::remove_file(root.join(STATE));
    let models = root.to_path_buf();
    let engine = crate::blocking(move || worker::Engine::load(&models)).await;
    let engine = match engine {
        Ok(engine) if engine.accelerated() => engine,
        failure => {
            let _ = fs::write(root.join(FAILED), b"");
            return Err(failure.err().unwrap_or_else(|| {
                AppError::model("Core ML reranking is unavailable; searches keep using the CPU")
            }));
        }
    };
    let _ = fs::remove_file(root.join(FAILED));
    let engine = Mutex::new(engine);
    let scorer: Scorer = Arc::new(move |query, documents| {
        engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .scores(query, documents)
    });
    let served = serve(root, scorer, IDLE).await;
    drop(lock);
    served
}

/// Serve on a free loopback port, publish where, and return once no request
/// has arrived for `idle`.
async fn serve(root: &Path, scorer: Scorer, idle: Duration) -> Result<(), AppError> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(AppError::io)?;
    let address = listener.local_addr().map_err(AppError::io)?;
    let mut random = [0u8; 32];
    getrandom::fill(&mut random).map_err(|_| AppError::internal("create worker token"))?;
    let token = blake3::Hash::from_bytes(random).to_hex().to_string();
    let requests = Arc::new(AtomicU64::new(0));
    let router = worker::router(&token, scorer, Arc::clone(&requests));
    publish(
        root,
        &State {
            url: format!("http://{address}/rerank"),
            token,
        },
    )?;
    let quiet = async move {
        let tick = (idle / 4).clamp(Duration::from_millis(10), Duration::from_secs(30));
        let mut seen = requests.load(Ordering::Relaxed);
        let mut since = Instant::now();
        loop {
            tokio::time::sleep(tick).await;
            let now = requests.load(Ordering::Relaxed);
            if now != seen {
                seen = now;
                since = Instant::now();
            } else if since.elapsed() >= idle {
                break;
            }
        }
    };
    let served = axum::serve(listener, router)
        .with_graceful_shutdown(quiet)
        .await
        .map_err(AppError::io);
    let _ = fs::remove_file(root.join(STATE));
    served
}

/// Write the state file whole, readable by this user only: it holds the token.
fn publish(root: &Path, state: &State) -> Result<(), AppError> {
    let temporary = root.join(format!("{STATE}.{}", std::process::id()));
    let _ = fs::remove_file(&temporary);
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    crate::platform::private_file(&mut options);
    let file = options.open(&temporary).map_err(AppError::io)?;
    serde_json::to_writer(&file, state)
        .map_err(|_| AppError::internal("cannot write the worker state"))?;
    file.sync_all().map_err(AppError::io)?;
    fs::rename(&temporary, root.join(STATE)).map_err(AppError::io)
}

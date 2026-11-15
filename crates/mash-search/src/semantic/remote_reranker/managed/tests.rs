use super::*;
use std::sync::atomic::AtomicUsize;

fn lengths() -> Scorer {
    Arc::new(|_, documents| {
        Ok(documents
            .iter()
            .map(|text| f32::from(u16::try_from(text.len()).unwrap()))
            .collect())
    })
}

/// Stands in for a live worker, which holds the lock for its whole life.
fn hold_lock(root: &Path) -> fs::File {
    let lock = lock_file(root).unwrap();
    lock.try_lock().unwrap();
    lock
}

fn counting(starts: &AtomicUsize) -> impl Fn(&Path) + '_ {
    |_| {
        starts.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn ordinary_command_lines_are_not_a_worker() {
    let args = |list: &[&str]| list.iter().map(OsString::from).collect::<Vec<_>>();
    assert_eq!(entry(args(&["mash", "search", "query"])), None);
    assert_eq!(entry(args(&["mash"])), None);
    assert_eq!(entry(args(&["mash", "search", ARGUMENT])), None);
    // The models directory is required.
    assert_eq!(entry(args(&["mash", ARGUMENT])), None);
}

#[test]
fn with_no_worker_one_is_started_and_the_search_uses_the_cpu() {
    let root = tempfile::tempdir().unwrap();
    let starts = AtomicUsize::new(0);
    assert!(connect_with(root.path(), &counting(&starts)).is_none());
    assert_eq!(starts.load(Ordering::SeqCst), 1);
}

#[test]
fn a_recent_failure_is_not_retried_on_every_search() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join(FAILED), b"").unwrap();
    let starts = AtomicUsize::new(0);
    assert!(connect_with(root.path(), &counting(&starts)).is_none());
    assert_eq!(starts.load(Ordering::SeqCst), 0);
}

#[test]
fn a_worker_that_is_still_loading_is_neither_used_nor_started_again() {
    let root = tempfile::tempdir().unwrap();
    let _worker = hold_lock(root.path());
    let starts = AtomicUsize::new(0);
    assert!(connect_with(root.path(), &counting(&starts)).is_none());
    assert_eq!(starts.load(Ordering::SeqCst), 0);
}

#[test]
fn a_state_file_left_by_a_dead_worker_is_not_trusted() {
    let root = tempfile::tempdir().unwrap();
    publish(
        root.path(),
        &State {
            url: "http://127.0.0.1:9/rerank".into(),
            token: "stale".into(),
        },
    )
    .unwrap();
    let starts = AtomicUsize::new(0);
    assert!(connect_with(root.path(), &counting(&starts)).is_none());
    assert_eq!(starts.load(Ordering::SeqCst), 1);
}

#[cfg(unix)]
#[test]
fn the_state_file_is_private_to_this_user() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    publish(
        root.path(),
        &State {
            url: "http://127.0.0.1:9/rerank".into(),
            token: "secret".into(),
        },
    )
    .unwrap();
    let mode = fs::metadata(root.path().join(STATE))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[tokio::test]
async fn a_search_finds_the_running_worker_and_it_leaves_when_idle() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().to_path_buf();
    let _worker = hold_lock(&path);
    let serving = tokio::spawn({
        let path = path.clone();
        async move { serve(&path, lengths(), Duration::from_millis(400)).await }
    });
    let starts = AtomicUsize::new(0);
    let client = loop {
        if let Some(client) = connect_with(&path, &counting(&starts)) {
            break client.unwrap();
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    };
    let scores = client.rerank("q", &["four", "sixsix"]).await.unwrap();
    assert_eq!(scores.scores, [4.0, 6.0]);
    // A second worker would have a different token; this one was never restarted.
    assert_eq!(starts.load(Ordering::SeqCst), 0);
    serving.await.unwrap().unwrap();
    assert!(!path.join(STATE).exists());
    assert!(connect_with(&path, &counting(&starts)).is_none());
}

#[tokio::test]
async fn requests_keep_the_worker_loaded() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().to_path_buf();
    let _worker = hold_lock(&path);
    let serving = tokio::spawn({
        let path = path.clone();
        async move { serve(&path, lengths(), Duration::from_millis(300)).await }
    });
    let client = loop {
        if let Some(client) = connect_with(&path, &|_| {}) {
            break client.unwrap();
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    };
    let started = Instant::now();
    // Warm-ups count as use, well past one idle period.
    while started.elapsed() < Duration::from_millis(900) {
        client
            .send(&super::super::Request::new("", &[]), TIMEOUT)
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(!serving.is_finished());
    serving.await.unwrap().unwrap();
}

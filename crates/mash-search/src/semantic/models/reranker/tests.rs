use super::*;

#[test]
fn settling_and_dropping_leave_no_load_running() {
    let missing = tempfile::tempdir().unwrap();
    let mut failed = Lazy::start(missing.path());
    assert!(failed.settle(), "a load from an empty directory fails");
    assert!(failed.loading.is_none());
    assert!(failed.get().is_err());

    let mut deferred = Lazy::deferred(missing.path());
    assert!(!deferred.settle(), "nothing was started, so nothing failed");
    assert!(deferred.loaded.is_none());

    // Dropping joins the loader: its result is gone when the drop returns.
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut dropped = Lazy::deferred(missing.path());
    dropped.loading = Some(std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = sender.send(());
        Err(AppError::internal("never loaded"))
    }));
    drop(dropped);
    assert!(receiver.try_recv().is_ok());
}

#[test]
#[ignore = "requires explicitly installed MASH_TEST_MODELS_DIR"]
fn reranker_cpu_scores_and_empty_inputs_preserve_contract() {
    let root = std::env::var_os("MASH_TEST_MODELS_DIR").unwrap();
    let selected = super::super::generations::select(Path::new(&root)).unwrap();
    let mut model = Reranker::load(&selected.root).unwrap();
    assert_eq!(model.run("query", &[]).unwrap().len(), 0);
    let documents = ["vector search in Qdrant", "garden carrots", "good morning"];
    let actual = model.run("vector search", &documents).unwrap();
    assert_eq!(actual.len(), documents.len());
    assert_eq!(actual[0].index, 0, "the relevant document must rank first");
    assert_eq!(
        actual
            .iter()
            .map(|item| item.index)
            .collect::<std::collections::BTreeSet<_>>(),
        (0..documents.len()).collect()
    );
    assert!(actual.iter().all(|item| item.score.is_finite()));
    assert!(actual.windows(2).all(|pair| pair[0].score >= pair[1].score));
}

//! Read-only checks of the last scan recorded for each searched provider.
use crate::provider::Provider;
use crate::silver::certificate::Completeness;
use crate::storage::Storage;
use std::path::Path;

pub(super) fn scan_warnings(database: &Path, providers: &[Provider]) -> Vec<String> {
    if providers.is_empty() {
        return Vec::new();
    }
    let Ok(storage) = Storage::open_silver(database) else {
        return vec![
            "Could not check the last local history scan. Run `mash doctor` for details.".into(),
        ];
    };
    let mut warnings = Vec::new();
    for &provider in providers {
        match storage.coverage_scan(provider) {
            Ok((Completeness::Complete, 0)) => {},
            Ok((state, rejected)) => warnings.push(format!(
                "Could not verify a complete local scan for {provider} ({state:?}, {rejected} rejected sources). Some chats may be missing. Run `mash index` and check its result."
            )),
            Err(_) => warnings.push(format!("Could not check the last local scan for {provider}. Run `mash doctor` for details.")),
        }
    }
    warnings
}

#[test]
fn freshness_inspection_never_creates_a_missing_database() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("missing.sqlite3");
    let warnings = scan_warnings(&database, &[Provider::Codex]);
    assert!(warnings[0].contains("Could not check the last local history scan"));
    assert!(!database.exists());
}

#[test]
fn freshness_warnings_require_missing_or_incomplete_scan_evidence() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("silver.sqlite3");
    let mut storage = Storage::open_writer(&database).unwrap();
    storage.begin_silver_scan(Provider::Codex).unwrap();
    storage.finish_silver_scan(Provider::Codex, true).unwrap();
    storage.begin_silver_scan(Provider::ClaudeCode).unwrap();
    storage
        .finish_silver_scan(Provider::ClaudeCode, false)
        .unwrap();
    storage.commit_writer().unwrap();
    drop(storage);

    assert_eq!(scan_warnings(&database, &[]), Vec::<String>::new());
    assert_eq!(
        scan_warnings(&database, &[Provider::Codex]),
        Vec::<String>::new()
    );
    let warnings = scan_warnings(
        &database,
        &[Provider::Codex, Provider::ClaudeCode, Provider::Copilot],
    );
    assert_eq!(warnings.len(), 2);
    assert!(warnings[0].contains("Incomplete") && warnings[0].contains("mash index"));
    assert!(warnings[1].contains("Unverified") && warnings[1].contains("mash index"));
}

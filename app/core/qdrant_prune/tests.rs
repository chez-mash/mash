use super::*;
use crate::storage::Storage;
use crate::views::qdrant::{MemoryPolicy, MemoryTier, Store, VectorIndex, collection};

fn local(database: &str) -> ownership::Owner {
    ownership::Owner::local(Path::new(database), Some("host".into()))
}

#[test]
fn only_a_marker_naming_a_gone_database_makes_a_leftover() {
    let in_use: BTreeSet<String> = ["used".to_owned()].into();
    let disk = |path: &Path| match path.to_str() {
        Some("/here/live.sqlite3") => OnDisk::Present,
        Some("/here/gone.sqlite3") => OnDisk::Gone,
        _ => OnDisk::Unreachable,
    };
    // An index this command was given wins over any marker.
    assert_eq!(
        classify("used", Some(&local("/here/gone.sqlite3")), &in_use, disk),
        Class::InUse
    );
    assert_eq!(classify("g", None, &in_use, disk), Class::Unknown);
    assert_eq!(
        classify("g", Some(&local("/here/live.sqlite3")), &in_use, disk),
        Class::InUse
    );
    assert_eq!(
        classify("g", Some(&local("/here/gone.sqlite3")), &in_use, disk),
        Class::Leftover
    );
    // An unmounted disk looks like a missing file; it is never a leftover.
    assert_eq!(
        classify("g", Some(&local("/mnt/off/x.sqlite3")), &in_use, disk),
        Class::OwnerUnreachable
    );
    let server = ownership::Owner::server("personal", None);
    assert_eq!(
        classify("g", Some(&server), &in_use, disk),
        Class::SharedServer
    );
}

#[test]
fn only_mash_views_pairs_are_considered() {
    let hash = "a".repeat(64);
    assert_eq!(
        group_of(&format!("mash_views_{hash}_messages")),
        Some(hash.as_str())
    );
    assert_eq!(
        group_of(&format!("mash_views_{hash}_state")),
        Some(hash.as_str())
    );
    for other in [
        "documents".to_owned(),
        format!("mash_views_{hash}_other"),
        format!("mash_views_{}_state", "a".repeat(63)),
        format!("mash_views_{}_state", "z".repeat(64)),
        format!("my_mash_views_{hash}_state"),
    ] {
        assert_eq!(group_of(&other), None, "{other}");
    }
}

#[test]
fn absence_counts_as_gone_only_in_a_readable_folder() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("index.sqlite3");
    std::fs::write(&file, b"").unwrap();
    assert_eq!(on_disk(&file), OnDisk::Present);
    assert_eq!(
        on_disk(&directory.path().join("gone.sqlite3")),
        OnDisk::Gone
    );
    assert_eq!(
        on_disk(&directory.path().join("no-folder/x.sqlite3")),
        OnDisk::Unreachable
    );
}

/// A real index database, and the views group indexing gives it.
fn index(directory: &Path, name: &str) -> PathBuf {
    let path = directory.join(name);
    let mut storage = Storage::open_writer(&path).unwrap();
    storage.commit_writer().unwrap();
    path
}

fn config(directory: &Path, url: &str) -> ResolvedConfig {
    let path = directory.join("config.toml");
    std::fs::write(
        &path,
        format!("version = 1\nproviders = {{}}\n[qdrant]\nurl = '{url}'\n"),
    )
    .unwrap();
    crate::config::load(Some(&path)).unwrap()
}

fn random() -> String {
    let mut bytes = [0u8; 12];
    getrandom::fill(&mut bytes).unwrap();
    hex::encode(bytes)
}

/// Create a group's collections exactly as indexing does.
async fn views(group: &str) -> Store {
    let endpoints = mash_model::test_support::qdrant::endpoints();
    let store = Store::connect(
        (&endpoints.url, endpoints.grpc_url()),
        None,
        &[group.to_owned()],
        4,
        64 * 1024 * 1024,
        VectorIndex::default(),
        MemoryPolicy {
            tier: MemoryTier::Cached,
            activate: true,
        },
    )
    .await
    .unwrap();
    let scope = crate::views::scope::Scope {
        group_id: group.to_owned(),
        node_id: "node".into(),
        generation: "fixture:test:4".to_owned(),
        dimensions: 4,
    };
    store.select_views(&scope, true).await.unwrap();
    store
}

fn hashed(group: &str) -> String {
    blake3::hash(group.as_bytes()).to_hex().to_string()
}

fn class_of(response: &Response, group: &str) -> Option<(Class, bool)> {
    let group = hashed(group);
    response
        .groups
        .iter()
        .find(|g| g.group == group)
        .map(|g| (g.class, g.remove))
}

/// A disposable Qdrant scene: two real indexes and one group of each kind.
struct Scene {
    directory: tempfile::TempDir,
    configuration: ResolvedConfig,
    this: PathBuf,
    other: PathBuf,
    this_group: String,
    other_group: String,
    leftover: String,
    live: String,
    unreachable: String,
    server: String,
    unknown: String,
    kept_unknown: String,
}

impl Scene {
    async fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let url = mash_model::test_support::qdrant::endpoints().url.clone();
        let configuration = config(directory.path(), &url);
        let this = index(directory.path(), "this.sqlite3");
        let other = index(directory.path(), "other.sqlite3");
        let tag = random();
        let scene = Self {
            this_group: crate::views::local::group_id(&identity(&this), "local"),
            other_group: crate::views::local::group_id(&identity(&other), "local"),
            leftover: format!("prune-{tag}-leftover"),
            live: format!("prune-{tag}-live"),
            unreachable: format!("prune-{tag}-unreachable"),
            server: format!("prune-{tag}-server"),
            unknown: format!("prune-{tag}-unknown"),
            kept_unknown: format!("prune-{tag}-kept-unknown"),
            directory,
            configuration,
            this,
            other,
        };
        for group in [
            &scene.this_group,
            &scene.other_group,
            &scene.unknown,
            &scene.kept_unknown,
        ] {
            views(group).await;
        }
        let gone = scene.directory.path().join("gone.sqlite3");
        let live = std::fs::canonicalize(&scene.this).unwrap();
        let unmounted = Path::new("/no-such-mount/x.sqlite3");
        for (group, owner) in [
            (&scene.leftover, ownership::Owner::local(&gone, None)),
            (&scene.live, ownership::Owner::local(&live, None)),
            (&scene.unreachable, ownership::Owner::local(unmounted, None)),
            (&scene.server, ownership::Owner::server(&scene.server, None)),
        ] {
            views(group)
                .await
                .record_owner(group, &owner)
                .await
                .unwrap();
        }
        scene
    }

    fn groups(&self) -> [&String; 8] {
        [
            &self.this_group,
            &self.other_group,
            &self.leftover,
            &self.live,
            &self.unreachable,
            &self.server,
            &self.unknown,
            &self.kept_unknown,
        ]
    }
}

/// A database's gold identity, as indexing computes it.
fn identity(path: &Path) -> String {
    let connection = rusqlite::Connection::open(path).unwrap();
    let instance: String = connection
        .query_row("SELECT instance FROM local_database_identity", [], |row| {
            row.get(0)
        })
        .unwrap();
    let canonical = std::fs::canonicalize(path).unwrap();
    crate::storage::gold_input::gold_identity(&instance, &canonical)
}

#[tokio::test]
async fn qdrant_prune_lists_every_kind_and_deletes_nothing_without_yes() {
    let scene = Scene::new().await;
    let request = Request {
        databases: vec![scene.other.clone()],
        ..Request::default()
    };
    let dry = run(request.clone(), &scene.this, &scene.configuration)
        .await
        .unwrap();
    assert!(!dry.applied);
    assert_eq!(dry.deleted.len(), 0);
    assert_eq!(
        class_of(&dry, &scene.this_group),
        Some((Class::InUse, false))
    );
    assert_eq!(
        class_of(&dry, &scene.other_group),
        Some((Class::InUse, false))
    );
    assert_eq!(class_of(&dry, &scene.live), Some((Class::InUse, false)));
    assert_eq!(
        class_of(&dry, &scene.leftover),
        Some((Class::Leftover, true))
    );
    assert_eq!(
        class_of(&dry, &scene.unreachable),
        Some((Class::OwnerUnreachable, false))
    );
    assert_eq!(
        class_of(&dry, &scene.server),
        Some((Class::SharedServer, false))
    );
    assert_eq!(
        class_of(&dry, &scene.unknown),
        Some((Class::Unknown, false))
    );
    // A name that is not a group here stops the command before anything is deleted.
    let wrong = Request {
        unknown: vec!["f".repeat(64)],
        apply: true,
        only: Some(BTreeSet::new()),
        ..request
    };
    assert!(run(wrong, &scene.this, &scene.configuration).await.is_err());
}

#[tokio::test]
async fn qdrant_prune_removes_only_leftovers_and_named_unknown_groups() {
    let scene = Scene::new().await;
    let apply = Request {
        apply: true,
        databases: vec![scene.other.clone()],
        unknown: vec![hashed(&scene.unknown)],
        only: Some(scene.groups().iter().map(|group| hashed(group)).collect()),
    };
    let applied = run(apply, &scene.this, &scene.configuration).await.unwrap();
    let removed: BTreeSet<_> = applied.deleted.iter().cloned().collect();
    let url = mash_model::test_support::qdrant::endpoints().url.clone();
    let names = api::Client::new(&url, None, 1024 * 1024)
        .unwrap()
        .collection_names()
        .await
        .unwrap();
    for group in [&scene.leftover, &scene.unknown] {
        assert!(removed.contains(&collection(group, false)), "{group}");
        assert!(removed.contains(&collection(group, true)), "{group}");
        assert!(
            !names.iter().any(|name| name.contains(&hashed(group))),
            "{group} is gone"
        );
    }
    let kept = [
        &scene.this_group,
        &scene.other_group,
        &scene.live,
        &scene.unreachable,
        &scene.server,
        &scene.kept_unknown,
    ];
    for group in kept {
        assert!(names.contains(&collection(group, true)), "{group} was kept");
    }
}

#[tokio::test]
async fn qdrant_prune_stops_when_a_leftover_gains_an_owner_and_deletes_state_last() {
    let directory = tempfile::tempdir().unwrap();
    let url = mash_model::test_support::qdrant::endpoints().url.clone();
    let client = api::Client::new(&url, None, 1024 * 1024).unwrap();
    let group = format!("prune-{}-restored", random());
    let store = views(&group).await;
    let gone = directory.path().join("restored.sqlite3");
    store
        .record_owner(&group, &ownership::Owner::local(&gone, None))
        .await
        .unwrap();
    let collections = vec![collection(&group, false), collection(&group, true)];
    let listed = Group {
        group: hashed(&group),
        collections,
        points: 0,
        class: Class::Leftover,
        owner: None,
        remove: true,
    };
    // The database comes back between the listing and the deletion.
    std::fs::write(&gone, b"").unwrap();
    let mut deleted = Vec::new();
    assert!(
        remove(&client, &listed, &BTreeSet::new(), &mut deleted)
            .await
            .is_err()
    );
    assert_eq!(deleted.len(), 0);
    std::fs::remove_file(&gone).unwrap();
    remove(&client, &listed, &BTreeSet::new(), &mut deleted)
        .await
        .unwrap();
    // Documents first, the marker's state collection last.
    assert_eq!(
        deleted,
        [collection(&group, false), collection(&group, true)]
    );
}

#[tokio::test]
async fn qdrant_prune_fails_closed_when_qdrant_does_not_answer() {
    let directory = tempfile::tempdir().unwrap();
    let configuration = config(directory.path(), "http://127.0.0.1:9");
    let this = index(directory.path(), "this.sqlite3");
    let request = Request {
        apply: true,
        only: Some(BTreeSet::new()),
        ..Request::default()
    };
    assert!(run(request, &this, &configuration).await.is_err());
}

#[tokio::test]
async fn qdrant_an_owner_marker_reads_back_and_is_told_apart_from_views() {
    let group = format!("prune-{}-marker", random());
    let store = views(&group).await;
    let state = collection(&group, true);
    let url = mash_model::test_support::qdrant::endpoints().url.clone();
    let client = api::Client::new(&url, None, 1024 * 1024).unwrap();
    assert_eq!(ownership::read_owner(&client, &state).await.unwrap(), None);
    let owner = ownership::Owner::server("personal", Some("xenia".into()));
    store.record_owner(&group, &owner).await.unwrap();
    // Writing it again changes nothing.
    store.record_owner(&group, &owner).await.unwrap();
    assert_eq!(
        ownership::read_owner(&client, &state).await.unwrap(),
        Some(owner)
    );
}

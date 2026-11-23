use super::*;
use serde_json::json;

fn row(chat: &str, node: Option<&str>) -> serde_json::Value {
    let mut row = json!({ "chat_id": chat, "provider": "codex", "score": 1.0, "matches": [] });
    if let Some(node) = node {
        row["node_id"] = json!(node);
    }
    row
}

fn ids(rows: &[serde_json::Value]) -> Vec<String> {
    rows.iter()
        .map(|row| {
            format!(
                "{}/{}",
                row["node_id"].as_str().unwrap_or("-"),
                row["chat_id"].as_str().unwrap()
            )
        })
        .collect()
}

#[test]
fn a_chat_found_by_several_sources_is_one_row_that_says_which() {
    let answered = vec![
        (Source::Words, vec![row("a", None), row("b", None)]),
        (Source::Meaning, vec![row("b", None), row("c", None)]),
        // This machine's own published copy of `a`, and another machine's chat.
        (
            Source::Machines,
            vec![row("a", Some("laptop")), row("d", Some("xenia"))],
        ),
    ];
    let fused = fuse("laptop", &answered, RANK);
    assert_eq!(fused.len(), 4, "{fused:?}");
    let a = fused.iter().find(|row| row["chat_id"] == "a").unwrap();
    // The local copy (no node) is kept: it was listed first.
    assert!(a.get("node_id").is_none());
    assert_eq!(a["matched_by"], json!(["words", "machines"]));
    let c = fused.iter().find(|row| row["chat_id"] == "c").unwrap();
    assert_eq!(c["matched_by"], json!(["meaning"]));
}

#[test]
fn chats_several_sources_agree_on_rank_above_one_sources_top_hit() {
    let answered = vec![
        (
            Source::Words,
            vec![row("only-words", None), row("both", None)],
        ),
        (
            Source::Meaning,
            vec![row("only-meaning", None), row("both", None)],
        ),
    ];
    let fused = fuse("laptop", &answered, RANK);
    assert_eq!(fused[0]["chat_id"], "both");
    // Equal scores keep the fixed source order: words before meaning.
    assert_eq!(ids(&fused[1..]), ["-/only-words", "-/only-meaning"]);
}

#[test]
fn the_ranking_does_not_depend_on_which_source_answered_first() {
    let words = (Source::Words, vec![row("a", None), row("b", None)]);
    let machines = (
        Source::Machines,
        vec![row("x", Some("xenia")), row("b", None)],
    );
    let mut first = vec![words.clone(), machines.clone()];
    let mut second = vec![machines, words];
    // `search` sorts answered sources before fusing; do the same here.
    first.sort_by_key(|(source, _)| *source);
    second.sort_by_key(|(source, _)| *source);
    assert_eq!(fuse("laptop", &first, RANK), fuse("laptop", &second, RANK));
}

#[test]
fn the_same_chat_id_on_two_machines_is_two_chats() {
    let answered = vec![(
        Source::Machines,
        vec![row("same", Some("xenia")), row("same", Some("build-box"))],
    )];
    assert_eq!(fuse("laptop", &answered, RANK).len(), 2);
}

#[test]
fn a_row_without_a_chat_id_is_left_out() {
    let answered = vec![(
        Source::Words,
        vec![json!({ "message_id": "m" }), row("a", None)],
    )];
    assert_eq!(ids(&fuse("laptop", &answered, RANK)), ["-/a"]);
}

fn spec(scope: Scope, query: &str) -> Spec {
    Spec {
        query: query.into(),
        limit: 10,
        filters: crate::search_filter::Selection::default(),
        scope,
        live: true,
        meaning: true,
        local_index: true,
    }
}

fn states(plan: &[(SourceStatus, Option<Operation>)]) -> Vec<(Source, bool)> {
    plan.iter()
        .map(|(status, operation)| (status.source, operation.is_some()))
        .collect()
}

#[test]
fn all_machines_asks_every_source() {
    let plan = plan(
        &spec(Scope::All, "invoice exports"),
        "invoice exports",
        10,
        "laptop",
        true,
        true,
    );
    assert_eq!(
        states(&plan),
        [
            (Source::Words, true),
            (Source::Meaning, true),
            (Source::Machines, true)
        ]
    );
    assert!(matches!(plan[0].1, Some(Operation::LiveSearch { .. })));
    assert!(matches!(
        plan[2].1,
        Some(Operation::Search { shared: true, .. })
    ));
}

#[test]
fn this_machine_never_reaches_the_network() {
    let plan = plan(
        &spec(Scope::ThisMachine, "invoice"),
        "invoice",
        10,
        "laptop",
        true,
        true,
    );
    assert!(plan[2].1.is_none());
    assert!(matches!(plan[2].0.state, State::Skipped { .. }));
    // Choosing this machine by its own ID is the same choice.
    let plan = plan_for(Scope::Machine("laptop".into()));
    assert!(plan[0].1.is_some() && plan[2].1.is_none());
}

fn plan_for(scope: Scope) -> Vec<(SourceStatus, Option<Operation>)> {
    plan(&spec(scope, "invoice"), "invoice", 10, "laptop", true, true)
}

#[test]
fn one_other_machine_searches_only_its_published_copy() {
    let plan = plan_for(Scope::Machine("xenia".into()));
    assert_eq!(
        states(&plan),
        [
            (Source::Words, false),
            (Source::Meaning, false),
            (Source::Machines, true)
        ]
    );
    assert_eq!(plan[2].0.label, "xenia");
    let Some(Operation::Search { filters, .. }) = &plan[2].1 else {
        panic!("a shared search")
    };
    assert_eq!(filters.node, ["xenia"]);
}

#[test]
fn meaning_waits_for_three_characters_a_caller_that_asks_and_dense_views() {
    for (query, asked, dense) in [
        ("ab", true, true),
        ("abc", false, true),
        ("abcd", true, false),
    ] {
        let mut wanted = spec(Scope::All, query);
        wanted.meaning = asked;
        let plan = plan(&wanted, query, 10, "laptop", true, dense);
        assert!(plan[1].1.is_none(), "{query} {asked} {dense}");
        assert!(matches!(plan[1].0.state, State::Skipped { .. }));
    }
}

#[test]
fn a_shared_only_dashboard_asks_only_the_shared_index() {
    let mut shared_only = spec(Scope::All, "invoice");
    shared_only.local_index = false;
    let plan = plan(&shared_only, "invoice", 10, "server", true, true);
    assert_eq!(
        states(&plan),
        [
            (Source::Words, false),
            (Source::Meaning, false),
            (Source::Machines, true)
        ]
    );
}

#[test]
fn an_unjoined_machine_skips_other_machines_and_says_why() {
    let plan = plan(
        &spec(Scope::All, "invoice"),
        "invoice",
        10,
        "local",
        false,
        false,
    );
    let State::Skipped { reason } = &plan[2].0.state else {
        panic!("skipped")
    };
    assert!(reason.contains("hasn't joined"));
}

#[test]
fn the_full_search_uses_full_word_search_and_longer_limits() {
    let mut full = spec(Scope::ThisMachine, "invoice");
    full.live = false;
    let plan = plan(&full, "invoice", 50, "laptop", false, false);
    assert!(matches!(
        plan[0].1,
        Some(Operation::Search {
            local: true,
            limit: Some(50),
            ..
        })
    ));
    assert!(time_limit(Source::Meaning, false) > time_limit(Source::Meaning, true));
}

#[test]
fn updates_serialize_as_one_json_object_per_line() {
    let update = update(
        "invoice",
        "laptop",
        &[(Source::Words, vec![row("a", None)])],
        &[SourceStatus {
            source: Source::Machines,
            label: "xenia".into(),
            state: State::Failed {
                message: "didn't answer in 3 s".into(),
                elapsed_ms: 3000,
                timed_out: true,
            },
        }],
        10,
        RANK,
    );
    let line = serde_json::to_string(&update).unwrap();
    assert!(!line.contains('\n'));
    let value: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(value["sources"][0]["state"], "failed");
    assert_eq!(value["sources"][0]["timed_out"], true);
    assert_eq!(value["results"][0]["matched_by"], json!(["words"]));
    assert_eq!(value["done"], true);
}

fn searching(source: Source) -> SourceStatus {
    SourceStatus {
        source,
        label: format!("{source:?}"),
        state: State::Searching,
    }
}

type Job = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Rows, AppError>>>>;

fn after(millis: u64, rows: Result<Rows, AppError>) -> Job {
    Box::pin(async move {
        tokio::time::sleep(Duration::from_millis(millis)).await;
        rows
    })
}

/// The default `[views.query.fusion]` rank constant.
const RANK: u16 = 60;

#[test]
fn federated_fusion_uses_the_configured_constant() {
    assert_eq!(
        RANK,
        crate::views::config::Fusion::default().rank_constant,
        "the tests use the default constant"
    );
}

fn answer() -> Answer {
    Answer {
        query: "invoice".into(),
        local_node: "laptop".into(),
        limit: 10,
        rank_constant: RANK,
    }
}

// Paused time: the clock advances past the 10 s source without waiting for it.
#[tokio::test(start_paused = true)]
async fn a_slow_source_is_reported_without_holding_back_the_others() {
    let statuses = vec![searching(Source::Words), searching(Source::Machines)];
    let jobs: Vec<(Source, Duration, Job)> = vec![
        (
            Source::Words,
            Duration::from_secs(1),
            after(5, Ok((vec![row("a", None)], None))),
        ),
        // Never answers in time.
        (
            Source::Machines,
            Duration::from_millis(50),
            after(10_000, Ok((vec![], None))),
        ),
    ];
    let started = tokio::time::Instant::now();
    let updates: Vec<Update> = drive(answer(), statuses, jobs).collect().await;
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "the slow source was not waited for"
    );
    assert_eq!(updates.len(), 3);
    // The words answered first and showed at once, before the timeout.
    assert_eq!(updates[1].results.len(), 1);
    assert!(!updates[1].done);
    let last = updates.last().unwrap();
    assert!(last.done);
    assert!(matches!(
        &last.sources[1].state,
        State::Failed { timed_out: true, message, .. } if message.contains("didn't answer")
    ));
    assert_eq!(last.results.len(), 1, "the words' rows stay");
}

#[tokio::test(start_paused = true)]
async fn a_failing_source_says_why_and_a_shared_error_keeps_its_rows() {
    let statuses = vec![searching(Source::Words), searching(Source::Machines)];
    let jobs: Vec<(Source, Duration, Job)> = vec![
        (
            Source::Words,
            Duration::from_secs(1),
            after(1, Err(AppError::usage("bad words"))),
        ),
        (
            Source::Machines,
            Duration::from_secs(1),
            after(
                5,
                Ok((
                    vec![row("x", Some("xenia"))],
                    Some("xenia is offline".into()),
                )),
            ),
        ),
    ];
    let updates: Vec<Update> = drive(answer(), statuses, jobs).collect().await;
    let last = updates.last().unwrap();
    assert!(
        matches!(&last.sources[0].state, State::Failed { message, timed_out: false, .. } if message == "bad words")
    );
    assert!(
        matches!(&last.sources[1].state, State::Failed { message, .. } if message == "xenia is offline")
    );
    assert_eq!(last.results.len(), 1);
}

#[tokio::test(start_paused = true)]
async fn a_reader_who_stops_listening_stops_the_search_and_its_sources() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    /// Records that the source it lives in was dropped before it finished.
    struct Probe(Arc<AtomicBool>);
    impl Drop for Probe {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let finished = Arc::new(AtomicBool::new(false));
    let dropped = Arc::new(AtomicBool::new(false));
    let slow = {
        let finished = finished.clone();
        let probe = Probe(dropped.clone());
        Box::pin(async move {
            let _probe = probe;
            tokio::time::sleep(Duration::from_millis(200)).await;
            finished.store(true, Ordering::SeqCst);
            Ok((vec![], None))
        }) as Job
    };
    let statuses = vec![searching(Source::Words), searching(Source::Meaning)];
    let jobs: Vec<(Source, Duration, Job)> = vec![
        (
            Source::Words,
            Duration::from_secs(1),
            after(1, Ok((vec![row("a", None)], None))),
        ),
        (Source::Meaning, Duration::from_secs(1), slow),
    ];
    // The reader leaves after the first answer (a newer query replaced this one).
    let mut updates = Box::pin(drive(answer(), statuses, jobs));
    assert!(updates.next().await.is_some(), "the opening update");
    assert_eq!(updates.next().await.unwrap().results.len(), 1);
    assert!(
        !dropped.load(Ordering::SeqCst),
        "the slow source is still running"
    );
    drop(updates);
    assert!(
        dropped.load(Ordering::SeqCst),
        "dropping the stream drops the sources still running"
    );
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(
        !finished.load(Ordering::SeqCst),
        "a cancelled search's sources must not keep working"
    );
}

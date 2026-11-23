//! Federated search: one question asked of several sources at once — this
//! machine's words, its search by meaning, and the other machines in the
//! viewing scope — merged into one ranked list that updates as each source
//! answers.
//!
//! Each source is an existing search operation run unchanged, so its own
//! ranking (lexical scoring, dense fusion, reranking) is what it reports. The
//! sources are merged by reciprocal rank fusion over each source's own order,
//! which needs no calibration between their incomparable scores, and a chat
//! found by several sources is one row. Sources are folded in a fixed order
//! whatever order they answer in, so the ranking never depends on timing.
//! A source that fails or outlives its time limit is reported, not retried,
//! and never holds back the others.
use crate::AppError;
use crate::core::{Operation, Paths, Request, Response};
use futures_util::stream::{FuturesUnordered, Stream, StreamExt};
use std::time::{Duration, Instant};

mod fusion;
#[cfg(test)]
mod tests;

pub use fusion::fuse;

/// Whose history is searched; the dashboard's viewing scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    /// This machine's own index only.
    ThisMachine,
    /// This machine's own index and every machine in the shared index.
    All,
    /// One other machine's published copy.
    Machine(String),
}

/// One kind of source. The order here is the order sources are folded in.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[schemars(rename = "SearchSource")]
pub enum Source {
    /// This machine's index, by words.
    Words,
    /// This machine's index, by meaning (dense retrieval and reranking).
    Meaning,
    /// The shared index: other machines' published chats.
    Machines,
}

/// What a search asks for.
#[derive(Debug, Clone)]
pub struct Spec {
    pub query: String,
    /// The most rows each update carries.
    pub limit: usize,
    pub filters: crate::search_filter::Selection,
    pub scope: Scope,
    /// Searching while the reader types: the fast word search and short time limits.
    pub live: bool,
    /// Also search by meaning. Callers turn it off while the reader is still
    /// typing a short query; it is also off below three characters.
    pub meaning: bool,
    /// This process has an index of its own; a dashboard that reads only the
    /// shared index does not, so it asks only the shared index.
    pub local_index: bool,
}

/// Where one source has got to.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum State {
    Searching,
    Done {
        count: usize,
        elapsed_ms: u64,
    },
    /// Failed or did not answer in time; `timed_out` tells the two apart.
    Failed {
        message: String,
        elapsed_ms: u64,
        timed_out: bool,
    },
    /// Not asked: out of scope, or not set up on this machine.
    Skipped {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub struct SourceStatus {
    pub source: Source,
    /// Who the source is, in words: "laptop", "by meaning", "your other machines", "xenia".
    pub label: String,
    #[serde(flatten)]
    pub state: State,
}

/// The merged ranking so far, and every source's state.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct Update {
    pub query: String,
    /// This machine's ID: a row without `node_id` is this machine's.
    pub local_node: String,
    /// Chat rows in merged order, as `mash search` returns them, each with
    /// `matched_by`: the sources that found it.
    pub results: Vec<serde_json::Value>,
    pub sources: Vec<SourceStatus>,
    /// No source is still searching.
    pub done: bool,
}

/// How long a source may take before it is reported as not answering.
pub fn time_limit(source: Source, live: bool) -> Duration {
    let millis = match (source, live) {
        (Source::Words, true) => 1_000,
        (Source::Meaning, true) => 2_000,
        (Source::Machines, true) => 3_000,
        (Source::Words, false) => 5_000,
        (Source::Meaning | Source::Machines, false) => 10_000,
    };
    Duration::from_millis(millis)
}

/// The most rows one live update carries: a reader still typing looks at a
/// handful, and the live word search returns at most 20.
const LIVE_LIMIT: usize = 20;
const FULL_LIMIT: usize = 100;
const MEANING_MIN_CHARS: usize = 3;

/// Run `spec` as a stream of updates: one when the search starts and one after
/// each source answers. The stream owns its sources, so dropping it (the reader
/// typed on, closed the page or the pipe) cancels the ones still running. What
/// stops a search before it starts (an empty query, unreadable configuration)
/// is returned as the error instead.
pub async fn search(
    paths: Paths,
    spec: Spec,
    capacity: Option<std::sync::Arc<tokio::sync::Semaphore>>,
) -> Result<impl Stream<Item = Update> + Send + 'static, AppError> {
    let query = spec.query.trim().to_owned();
    if query.is_empty() {
        return Err(AppError::usage("search needs a query"));
    }
    let limit = spec
        .limit
        .clamp(1, if spec.live { LIVE_LIMIT } else { FULL_LIMIT });
    let path = paths.config.clone();
    let configuration = crate::blocking(move || crate::config::load(path.as_deref())).await?;
    let local_node = configuration
        .shared
        .as_ref()
        .map_or_else(|| "local".to_owned(), |shared| shared.node_id.clone());
    let joined = configuration.shared.is_some();
    let dense = configuration.views.build.dense_enabled();
    let plan = plan(&spec, &query, limit, &local_node, joined, dense);
    let statuses: Vec<SourceStatus> = plan.iter().map(|(status, _)| status.clone()).collect();
    let jobs = plan
        .into_iter()
        .filter_map(|(status, operation)| operation.map(|operation| (status.source, operation)))
        .map(|(source, operation)| {
            let request = Request {
                paths: paths.clone(),
                operation,
            };
            (
                source,
                time_limit(source, spec.live),
                run(request, capacity.clone()),
            )
        })
        .collect();
    let answer = Answer {
        query,
        local_node,
        limit,
        rank_constant: configuration.views.query.fusion.rank_constant,
    };
    Ok(drive(answer, statuses, jobs))
}

/// What every update repeats.
pub(crate) struct Answer {
    pub(crate) query: String,
    pub(crate) local_node: String,
    pub(crate) limit: usize,
    /// The configured `[views.query.fusion]` constant, as every other merge uses.
    pub(crate) rank_constant: u16,
}

type Rows = (Vec<serde_json::Value>, Option<String>);

/// Run every source at once, each within its time limit, yielding the merged
/// ranking when the search starts and after each source answers. The sources
/// live inside the stream, so dropping it drops the ones still running.
pub(crate) fn drive<F>(
    answer: Answer,
    statuses: Vec<SourceStatus>,
    jobs: Vec<(Source, Duration, F)>,
) -> impl Stream<Item = Update>
where
    F: std::future::Future<Output = Result<Rows, AppError>>,
{
    async_stream::stream! {
        let Answer { query, local_node, limit, rank_constant } = answer;
        let mut statuses = statuses;
        yield update(&query, &local_node, &[], &statuses, limit, rank_constant);
        let mut pending = jobs
            .into_iter()
            .map(|(source, allowed, job)| async move {
                let started = Instant::now();
                let answer = tokio::time::timeout(allowed, job).await;
                (source, allowed, started.elapsed(), answer)
            })
            .collect::<FuturesUnordered<_>>();
        let mut answered: Vec<(Source, Vec<serde_json::Value>)> = Vec::new();
        while let Some((source, allowed, elapsed, answer)) = pending.next().await {
            let state = settle(&mut answered, source, allowed, elapsed, answer);
            if let Some(status) = statuses.iter_mut().find(|status| status.source == source) {
                status.state = state;
            }
            answered.sort_by_key(|(source, _)| *source);
            yield update(&query, &local_node, &answered, &statuses, limit, rank_constant);
        }
    }
}

/// One source's answer as its state, keeping any rows it brought.
fn settle(
    answered: &mut Vec<(Source, Vec<serde_json::Value>)>,
    source: Source,
    allowed: Duration,
    elapsed: Duration,
    answer: Result<Result<Rows, AppError>, tokio::time::error::Elapsed>,
) -> State {
    let elapsed_ms = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX);
    match answer {
        Err(_) => State::Failed {
            message: format!("didn't answer in {} s", allowed.as_secs_f32()),
            elapsed_ms,
            timed_out: true,
        },
        Ok(Err(error)) => State::Failed {
            message: error.error.message,
            elapsed_ms,
            timed_out: false,
        },
        Ok(Ok((rows, None))) => {
            let count = rows.len();
            answered.push((source, rows));
            State::Done { count, elapsed_ms }
        }
        // A shared search that could not reach the server still answers,
        // with the server's error beside whatever it had.
        Ok(Ok((rows, Some(message)))) => {
            answered.push((source, rows));
            State::Failed {
                message,
                elapsed_ms,
                timed_out: false,
            }
        }
    }
}

/// Run one source's search, keeping its chat rows and any shared error.
async fn run(
    request: Request,
    capacity: Option<std::sync::Arc<tokio::sync::Semaphore>>,
) -> Result<Rows, AppError> {
    let response = match capacity {
        Some(capacity) => crate::core::execute_with_model_limit(request, &capacity).await?,
        None => crate::core::execute(request).await?,
    };
    let Response::Search(response) = response else {
        return Err(AppError::internal("search answered with something else"));
    };
    let crate::search_results::Results::Chats(chats) = response.results else {
        return Ok((Vec::new(), response.shared_error));
    };
    let rows = chats
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| AppError::internal("serialize search results"))?;
    Ok((rows, response.shared_error))
}

fn update(
    query: &str,
    local_node: &str,
    answered: &[(Source, Vec<serde_json::Value>)],
    sources: &[SourceStatus],
    limit: usize,
    rank_constant: u16,
) -> Update {
    let mut results = fuse(local_node, answered, rank_constant);
    results.truncate(limit);
    Update {
        query: query.to_owned(),
        local_node: local_node.to_owned(),
        results,
        done: !sources
            .iter()
            .any(|status| status.state == State::Searching),
        sources: sources.to_vec(),
    }
}

/// Each source's opening status, and the operation to run unless it is skipped.
pub(crate) fn plan(
    spec: &Spec,
    query: &str,
    limit: usize,
    local_node: &str,
    joined: bool,
    dense: bool,
) -> Vec<(SourceStatus, Option<Operation>)> {
    // Another machine chosen by its ID that is in fact this one means this machine.
    let scope = match &spec.scope {
        Scope::Machine(node) if node == local_node => Scope::ThisMachine,
        scope => scope.clone(),
    };
    let here = spec.local_index && matches!(scope, Scope::ThisMachine | Scope::All);
    let status = |source, label: &str, state| SourceStatus {
        source,
        label: label.to_owned(),
        state,
    };
    let skip = |reason: &str| State::Skipped {
        reason: reason.to_owned(),
    };
    let words = if here {
        let operation = if spec.live {
            Operation::LiveSearch {
                query: query.to_owned(),
                limit: Some(limit.min(LIVE_LIMIT)),
                filters: Box::new(spec.filters.clone()),
            }
        } else {
            local_search(
                query,
                limit,
                &spec.filters,
                crate::views::exact::Intent::ExactMention,
            )
        };
        (
            status(Source::Words, local_node, State::Searching),
            Some(operation),
        )
    } else {
        (status(Source::Words, local_node, skip(away(spec))), None)
    };
    let short = query.chars().filter(|c| !c.is_whitespace()).count() < MEANING_MIN_CHARS;
    let meaning = if !here {
        (
            status(Source::Meaning, "by meaning", skip(away(spec))),
            None,
        )
    } else if !dense {
        let reason = "search by meaning isn't set up on this machine";
        (status(Source::Meaning, "by meaning", skip(reason)), None)
    } else if !spec.meaning || short {
        (
            status(
                Source::Meaning,
                "by meaning",
                skip("the query is still short"),
            ),
            None,
        )
    } else {
        let operation = local_search(
            query,
            limit,
            &spec.filters,
            crate::views::exact::Intent::RelatedDiscussion,
        );
        (
            status(Source::Meaning, "by meaning", State::Searching),
            Some(operation),
        )
    };
    let machines = match (&scope, joined) {
        (Scope::ThisMachine, _) => {
            let reason = "only this machine is chosen";
            (
                status(Source::Machines, "your other machines", skip(reason)),
                None,
            )
        }
        (_, false) => {
            let reason = "this machine hasn't joined a shared index";
            (
                status(Source::Machines, "your other machines", skip(reason)),
                None,
            )
        }
        (Scope::All, true) => (
            status(Source::Machines, "your other machines", State::Searching),
            Some(shared_search(query, limit, spec.filters.clone())),
        ),
        (Scope::Machine(node), true) => {
            let mut filters = spec.filters.clone();
            filters.node = vec![node.clone()];
            (
                status(Source::Machines, node, State::Searching),
                Some(shared_search(query, limit, filters)),
            )
        }
    };
    vec![words, meaning, machines]
}

/// Why this machine's own index is not asked.
fn away(spec: &Spec) -> &'static str {
    if spec.local_index {
        "another machine is chosen"
    } else {
        "this dashboard reads only the shared index"
    }
}

fn local_search(
    query: &str,
    limit: usize,
    filters: &crate::search_filter::Selection,
    intent: crate::views::exact::Intent,
) -> Operation {
    Operation::Search {
        shared_vector_search: None,
        query: query.to_owned(),
        intent: Some(intent),
        limit: Some(limit),
        offset: 0,
        unit: Some(crate::search_results::SearchUnit::Chats),
        fusion: crate::views::config::FusionOverrides::default(),
        ranking: None,
        diagnostics: false,
        filters: Box::new(filters.clone()),
        shared: false,
        local: true,
    }
}

/// The shared index's own default search, as `mash search --shared` runs it.
fn shared_search(query: &str, limit: usize, filters: crate::search_filter::Selection) -> Operation {
    Operation::Search {
        shared_vector_search: None,
        query: query.to_owned(),
        intent: None,
        limit: Some(limit),
        offset: 0,
        unit: Some(crate::search_results::SearchUnit::Chats),
        fusion: crate::views::config::FusionOverrides::default(),
        ranking: None,
        diagnostics: false,
        filters: Box::new(filters),
        shared: true,
        local: false,
    }
}

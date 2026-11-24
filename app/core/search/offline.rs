//! Word search while Qdrant is not running.
//!
//! Search reads Qdrant, but the history it indexes is saved in SQLite first.
//! When Qdrant does not answer, a word search (`exact-mention`) on this
//! machine scans the saved messages instead, through
//! [`crate::storage::Storage::scan_messages`], and says so in its notice: the
//! results are newest first rather than ranked, and may stop short on a large
//! history. Search by meaning still needs Qdrant and fails as before, and so
//! does a search of only the shared index, which never reads this machine.
use super::{Input, SearchMode, SearchResponse, node};
use crate::AppError;
use crate::config::ResolvedConfig;
use crate::storage::Storage;
use std::path::Path;

/// What a word search asked for, kept in case Qdrant does not answer.
pub(super) struct Retry {
    query: String,
    limit: Option<usize>,
    offset: usize,
    unit: Option<crate::search_results::SearchUnit>,
    filters: crate::search_filter::Selection,
}

impl Retry {
    /// The retry for a search that may scan instead, or None for one that
    /// cannot: a search by meaning, or one of the shared index only.
    pub(super) fn new(input: &Input, configuration: &ResolvedConfig) -> Option<Self> {
        let intent = input.intent.unwrap_or(configuration.views.query.intent);
        (intent == crate::views::exact::Intent::ExactMention && !input.shared).then(|| Self {
            query: input.query.clone(),
            limit: input.limit,
            offset: input.offset,
            unit: input.unit,
            filters: input.filters.clone(),
        })
    }
}

const NOTICE: &str = "Qdrant is not running, so this searched the saved messages on this machine \
     directly: whole words, newest first, without ranking or other machines.";
const PARTIAL: &str = " The scan stopped early, so older matches may be missing.";
const START: &str = " Start Qdrant (`mash qdrant install`) for ranked search.";

pub(super) async fn search(
    configuration: &ResolvedConfig,
    database: &Path,
    retry: Retry,
) -> Result<SearchResponse, AppError> {
    let (page, unit) = configuration
        .views
        .query
        .resolve(retry.limit, retry.offset, retry.unit)?;
    let scope = retry.filters.resolve(node(configuration))?;
    let needles = crate::views::matching::tokens(&retry.query);
    if needles.is_empty() {
        return Err(AppError::usage("search needs at least one word"));
    }
    let path = database.to_path_buf();
    let filter = scope.clone();
    let mut scan =
        crate::blocking(move || Storage::open_silver(&path)?.scan_messages(&needles, &filter))
            .await?;
    // Every hit contains every word, so each is word evidence; scores fall
    // with age so that chats rank by their newest match.
    let mut newest = 1.0_f32;
    for hit in &mut scan.hits {
        hit.lexical_score = Some(1.0);
        hit.fusion_score = newest;
        newest = newest.next_down();
    }
    let output = configuration.redaction.output();
    let collected = crate::search_results::collect(
        scan.hits,
        unit,
        scope.subagents,
        page,
        &retry.query,
        output,
    );
    let mut notice = NOTICE.to_owned();
    if !scan.complete {
        notice.push_str(PARTIAL);
    }
    notice.push_str(START);
    Ok(SearchResponse {
        notice: Some(notice),
        freshness_warnings: Vec::new(),
        diagnostics: None,
        intent: crate::views::exact::Intent::ExactMention,
        realized_mode: SearchMode::Lexical,
        unit,
        results: collected.results,
        next_offset: collected.next_offset,
        query: retry.query,
        shared: false,
        shared_error: None,
        shared_context_error: None,
    })
}

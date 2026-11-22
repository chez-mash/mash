//! Word search (exact-mention by BM25, no models): the full search for
//! several unquoted words, and the live search run while the reader types.
use super::{SearchMode, SearchResponse, exact_mode, node, required_shared, resolve_fusion};
use crate::AppError;
use crate::config::ResolvedConfig;
use crate::shared;
use std::path::Path;

/// Exact-mention for several unquoted words: each required, ranked by BM25
/// and recency.
pub(super) async fn lexical(
    (configuration, database, use_shared, use_local): (&ResolvedConfig, &Path, bool, bool),
    query: String,
    (options, page): (crate::views::config::Options, crate::search_results::Page),
    filters: &crate::search_filter::Scope,
) -> Result<SearchResponse, AppError> {
    let database = (!use_shared).then_some(database);
    let shared = use_shared || (!use_local && configuration.shared.is_some());
    let (hits, shared_error) = shared::search_lexical(
        shared.then(|| required_shared(configuration)).transpose()?,
        database.map(|database| (database, configuration)),
        &query,
        options,
        filters,
    )
    .await?;
    let realized_mode = if shared {
        exact_mode(shared_error.is_some(), database.is_none())
    } else {
        SearchMode::Lexical
    };
    let shared = shared && shared_error.is_none();
    Ok(word_response(
        hits,
        (options, page),
        query,
        realized_mode,
        (shared, shared_error),
    ))
}

/// The most results one live search returns: a reader who is still typing
/// looks at a handful, and fewer rows cost less to read and mask.
const LIVE_LIMIT: usize = 20;
const LIVE_DEFAULT_LIMIT: usize = 8;

/// Search while the reader types (`Operation::LiveSearch`): word search over
/// this machine's index only, one short page of chats, and none of the full
/// search's slower work (the literal scan, shared nodes, freshness checks).
/// Pressing Enter runs the full search for the same words.
pub(in crate::core) async fn live(
    configuration: &ResolvedConfig,
    database: &Path,
    query: String,
    limit: Option<usize>,
    filters: crate::search_filter::Selection,
) -> Result<SearchResponse, AppError> {
    let limit = limit.unwrap_or(LIVE_DEFAULT_LIMIT);
    if !(1..=LIVE_LIMIT).contains(&limit) {
        return Err(AppError::usage(format!(
            "live search returns 1 to {LIVE_LIMIT} results"
        )));
    }
    let intent = crate::views::exact::Intent::ExactMention;
    configuration.views.build.require_intent(intent)?;
    crate::search_query::validate_words(&query)?;
    let (page, unit) = configuration.views.query.resolve(
        Some(limit),
        0,
        Some(crate::search_results::SearchUnit::Chats),
    )?;
    let mut options = crate::views::config::Options::new(unit);
    options.fusion = resolve_fusion(
        crate::views::config::FusionOverrides::default(),
        intent,
        configuration.views.query.fusion,
    )?;
    options.output_redaction = configuration.redaction.output();
    let filters = filters.resolve(node(configuration))?;
    let hits = shared::search_typing((database, configuration), &query, options, &filters).await?;
    Ok(word_response(
        hits,
        (options, page),
        query,
        SearchMode::Lexical,
        (false, None),
    ))
}

fn word_response(
    hits: Vec<crate::storage::SearchHit>,
    (options, page): (crate::views::config::Options, crate::search_results::Page),
    query: String,
    realized_mode: SearchMode,
    (shared, shared_error): (bool, Option<String>),
) -> SearchResponse {
    let collected =
        crate::search_results::collect(hits, options.unit, page, &query, options.output_redaction);
    SearchResponse {
        notice: None,
        freshness_warnings: Vec::new(),
        diagnostics: None,
        intent: crate::views::exact::Intent::ExactMention,
        realized_mode,
        unit: options.unit,
        results: collected.results,
        next_offset: collected.next_offset,
        query,
        shared,
        shared_error,
        shared_context_error: None,
    }
}

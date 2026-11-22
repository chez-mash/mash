//! Word search (exact-mention by BM25): over this machine, the shared nodes or
//! both for a full search, and over this machine alone while the reader types.
//! Both order their hits with one ranking.
use super::protocol;
use super::retrieval::{
    apply_projections, projected_documents, received_candidates, request_candidates,
};
use crate::storage::SearchHit;
use crate::views::config::Options;
use crate::{AppError, config::SharedConfig};
use std::path::Path;

/// Model-free word search over this machine (`local_database`), the shared
/// nodes (`config`), or both. A failed shared request keeps local results and
/// is reported; alone, it is the error.
pub async fn search_lexical(
    config: Option<&SharedConfig>,
    local_database: Option<(&Path, &crate::config::ResolvedConfig)>,
    query: &str,
    options: Options,
    filters: &crate::search_filter::Scope,
) -> Result<(Vec<SearchHit>, Option<String>), AppError> {
    filters.filter.validate()?;
    crate::search_query::validate_words(query)?;
    let exclude_self = local_database.is_some();
    let request = |generation: String, exclude_self| {
        word_request(query, options, filters, generation, exclude_self)
    };
    let local = async {
        let Some((path, configuration)) = local_database else {
            return Ok(Vec::new());
        };
        let local = crate::views::local::Local::open_words_current(path, configuration).await?;
        let input = request(local.scope.generation.clone(), false);
        let mut hits = local.candidates(input).await?.lexical;
        for hit in &mut hits {
            hit.node_id = None;
        }
        Ok::<_, AppError>(hits)
    };
    let remote = async {
        let Some(config) = config else {
            return Ok(None);
        };
        let input = request(String::new(), exclude_self);
        Ok::<_, AppError>(Some(request_candidates(config, &input).await))
    };
    let (mut hits, remote) = tokio::try_join!(local, remote)?;
    let mut error = None;
    if let Some(remote) = remote {
        let (candidates, retrieval) = received_candidates(remote, exclude_self)?;
        error = retrieval.into_error();
        if !candidates.semantic.is_empty()
            || candidates.lexical.iter().any(|hit| {
                hit.node_id.as_ref().is_none_or(String::is_empty)
                    || hit.lexical_score.is_none_or(|score| !score.is_finite())
            })
        {
            return Err(AppError::shared("invalid lexical response"));
        }
        let documents = projected_documents(candidates.documents);
        let mut shared = candidates.lexical;
        apply_projections(shared.iter_mut(), &documents);
        hits.extend(shared);
    }
    Ok((rank_words(hits, options), error))
}

/// Word search over this machine's index alone, for a reader still typing:
/// every query word by BM25, each also by its prefix and typo alternatives,
/// even for a single term (see `Store::search_typing`). It never contacts the
/// shared nodes, so a keystroke waits on nothing off this machine.
pub async fn search_typing(
    (path, configuration): (&Path, &crate::config::ResolvedConfig),
    query: &str,
    options: Options,
    filters: &crate::search_filter::Scope,
) -> Result<Vec<SearchHit>, AppError> {
    filters.filter.validate()?;
    crate::search_query::validate_words(query)?;
    let local = crate::views::local::Local::open_words_current(path, configuration).await?;
    let input = word_request(
        query,
        options,
        filters,
        local.scope.generation.clone(),
        false,
    );
    let mut hits = local.typing_candidates(input).await?.lexical;
    for hit in &mut hits {
        hit.node_id = None;
    }
    Ok(rank_words(hits, options))
}

fn word_request(
    query: &str,
    options: Options,
    filters: &crate::search_filter::Scope,
    generation: String,
    exclude_self: bool,
) -> protocol::Search {
    protocol::Search {
        match_mode: filters.match_mode,
        vector_policy: None,
        intent: crate::views::exact::Intent::ExactMention,
        query: query.to_owned(),
        unit: options.unit,
        vector: Vec::new(),
        generation,
        provider: None,
        days: None,
        filter: Some(filters.filter.clone()),
        exclude_self,
    }
}

/// Order word hits from any source the same way: best BM25 first, one per
/// chat or message, newer favoured, then spread across chats.
fn rank_words(mut hits: Vec<SearchHit>, options: Options) -> Vec<SearchHit> {
    // BM25 is negated on arrival, so the best match has the lowest score.
    hits.sort_by(|a, b| {
        a.lexical_score
            .unwrap_or(f64::INFINITY)
            .total_cmp(&b.lexical_score.unwrap_or(f64::INFINITY))
            .then_with(|| a.node_id.cmp(&b.node_id))
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut hits = crate::search_results::deduplicate(hits, options.unit);
    crate::search_ranking::favour_recent(&mut hits, options.fusion);
    crate::search_results::diversify(hits, options.unit)
}

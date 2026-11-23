//! Read-only history search and its query parameters: the full search, and
//! the live one the search box runs while the reader types.
use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::{extract::State, response::Response};

use super::{Dash, Operation, Request, core, json_result};

mod stream;
use mash_model::AppError;

#[derive(serde::Deserialize, Default, schemars::JsonSchema)]
#[serde(default)]
pub(super) struct SearchParams {
    pub(super) q: String,
    /// Comma-separated provider names, such as `claude-code,codex`.
    pub(super) provider: String,
    /// Comma-separated machine (node) IDs.
    pub(super) machine: String,
    /// Comma-separated recorded working directories.
    pub(super) project: String,
    pub(super) days: Option<u32>,
    /// Search intent override. Omission uses the machine's configured default.
    pub(super) intent: Option<crate::views::exact::Intent>,
    pub(super) offset: usize,
    /// relevance (default) or recent: newest first within each page of matching chats.
    pub(super) sort: crate::search_filter::Sort,
}

fn split(list: &str) -> Vec<String> {
    list.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

fn providers(list: &str) -> Result<Vec<crate::provider::Provider>, AppError> {
    split(list)
        .into_iter()
        .map(|name| {
            serde_json::from_value(serde_json::Value::String(name))
                .map_err(|_| AppError::usage("unknown provider"))
        })
        .collect()
}

/// A query searches messages; an empty one lists the matching chats, newest first.
pub(super) fn search_operation(
    params: SearchParams,
    shared_only: bool,
) -> Result<Operation, AppError> {
    let provider = providers(&params.provider)?;
    if params.q.trim().is_empty() {
        let input = crate::chat_catalog::Request {
            provider,
            node: split(&params.machine),
            ..Default::default()
        };
        return Ok(Operation::ListChats {
            input: Box::new(input),
            local: false,
            shared: shared_only,
            diagnostics: false,
        });
    }
    let filters = crate::search_filter::Selection {
        provider,
        node: split(&params.machine),
        project: split(&params.project),
        days: params.days,
        sort: params.sort,
        ..Default::default()
    };
    Ok(Operation::Search {
        shared_vector_search: None,
        query: params.q,
        intent: params.intent,
        limit: Some(10),
        offset: params.offset,
        unit: Some(crate::search_results::SearchUnit::Chats),
        fusion: crate::views::config::FusionOverrides::default(),
        ranking: None,
        diagnostics: false,
        filters: Box::new(filters),
        shared: shared_only,
        local: false,
    })
}

async fn search(
    State(state): State<Arc<Dash>>,
    axum::extract::Query(params): axum::extract::Query<SearchParams>,
) -> Response {
    match search_operation(params, state.shared_only) {
        Ok(operation) => json_result(
            core::execute_with_model_limit(
                Request {
                    paths: state.paths.clone(),
                    operation,
                },
                &state.model_work,
            )
            .await,
        ),
        Err(error) => json_result(Err(error)),
    }
}

/// The search routes.
pub(super) fn routes(router: ApiRouter<Arc<Dash>>) -> ApiRouter<Arc<Dash>> {
    use super::contract::reads;
    use aide::axum::routing::get_with;
    router
        .api_route(
            "/api/search",
            get_with(
                search,
                reads::<core::SearchResponse>("/api/search", "Chats that match a question."),
            ),
        )
        .api_route(
            "/api/search/stream",
            get_with(
                stream::stream,
                reads::<core::search::federated::Update>(
                    "/api/search/stream",
                    "A search of every source in the viewing scope, as lines of JSON: the merged ranking so far and each source's state, after each source answers.",
                ),
            ),
        )
        .api_route(
            "/api/search/live",
            get_with(
                live,
                reads::<core::SearchResponse>(
                    "/api/search/live",
                    "A few chats on this machine whose words match what the reader is typing.",
                ),
            ),
        )
}

#[derive(serde::Deserialize, Default, schemars::JsonSchema)]
#[serde(default)]
pub(super) struct LiveParams {
    pub(super) q: String,
    /// Comma-separated provider names, such as `claude-code,codex`.
    pub(super) provider: String,
    /// Comma-separated recorded working directories.
    pub(super) project: String,
    pub(super) days: Option<u32>,
    /// At most 20; 8 when omitted.
    pub(super) limit: Option<usize>,
}

/// Live search reads this machine's own index, so a dashboard that serves
/// only the shared index has none to read; it runs the full search instead.
pub(super) fn live_operation(params: LiveParams, shared_only: bool) -> Result<Operation, AppError> {
    if shared_only {
        return Err(AppError::usage(
            "live search reads this machine's index, which this dashboard does not serve",
        ));
    }
    let filters = crate::search_filter::Selection {
        provider: providers(&params.provider)?,
        project: split(&params.project),
        days: params.days,
        ..Default::default()
    };
    Ok(Operation::LiveSearch {
        query: params.q,
        limit: params.limit,
        filters: Box::new(filters),
    })
}

async fn live(
    State(state): State<Arc<Dash>>,
    axum::extract::Query(params): axum::extract::Query<LiveParams>,
) -> Response {
    let operation = live_operation(params, state.shared_only);
    json_result(match operation {
        Ok(operation) => {
            core::execute(Request {
                paths: state.paths.clone(),
                operation,
            })
            .await
        }
        Err(error) => Err(error),
    })
}

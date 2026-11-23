//! Federated search as a stream: one line of JSON per update, the merged
//! ranking so far and every source's state, as each source answers.
//!
//! The response body is the search's own stream, so when the browser stops
//! reading (it typed on or left the page) the body is dropped and the sources
//! still running go with it: a cancelled query neither keeps working nor
//! delivers its results to a newer one.
use super::{providers, split};
use crate::core::search::federated::{self, Scope, Spec};
use crate::{Dash, json_result};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use mash_model::AppError;
use std::sync::Arc;

#[derive(serde::Deserialize, Default, schemars::JsonSchema)]
#[serde(default)]
pub(crate) struct StreamParams {
    pub(crate) q: String,
    /// Comma-separated provider names, such as `claude-code,codex`.
    pub(crate) provider: String,
    /// Comma-separated recorded working directories.
    pub(crate) project: String,
    pub(crate) days: Option<u32>,
    /// Whose history: `local` (this machine), `all` (the default) or `node:<machine>`.
    pub(crate) scope: String,
    /// At most 20 while typing (`live`), 100 otherwise; 10 when omitted.
    pub(crate) limit: Option<usize>,
    /// Searching while the reader types: fast word search and short time limits.
    pub(crate) live: bool,
    /// Skip search by meaning, as a reader still typing a short query does.
    pub(crate) words_only: bool,
}

pub(crate) fn spec(params: StreamParams, shared_only: bool) -> Result<Spec, AppError> {
    let scope = match params.scope.as_str() {
        "" | "all" => Scope::All,
        "local" => Scope::ThisMachine,
        other => match other.strip_prefix("node:") {
            Some(node) if !node.is_empty() => Scope::Machine(node.to_owned()),
            _ => return Err(AppError::usage("scope is local, all or node:<machine>")),
        },
    };
    Ok(Spec {
        query: params.q,
        limit: params.limit.unwrap_or(10),
        filters: crate::search_filter::Selection {
            provider: providers(&params.provider)?,
            project: split(&params.project),
            days: params.days,
            ..Default::default()
        },
        scope,
        live: params.live,
        meaning: !params.words_only,
        local_index: !shared_only,
    })
}

pub(crate) async fn stream(
    State(state): State<Arc<Dash>>,
    axum::extract::Query(params): axum::extract::Query<StreamParams>,
) -> Response {
    let spec = match spec(params, state.shared_only) {
        Ok(spec) => spec,
        Err(error) => return json_result(Err(error)),
    };
    let search = federated::search(state.paths.clone(), spec, Some(state.model_work.clone()));
    // A search that could not start says why, as its only line.
    let body = futures_util::stream::once(search).flat_map(|started| match started {
        Ok(updates) => updates.map(|update| line(&update)).left_stream(),
        Err(error) => futures_util::stream::iter([line(&error)]).right_stream(),
    });
    let body = body.map(Ok::<_, std::convert::Infallible>);
    (
        [
            (axum::http::header::CONTENT_TYPE, "application/x-ndjson"),
            (axum::http::header::CACHE_CONTROL, "no-store"),
        ],
        axum::body::Body::from_stream(body),
    )
        .into_response()
}

/// An update or an error as one line of JSON.
fn line(value: &impl serde::Serialize) -> axum::body::Bytes {
    let mut line = serde_json::to_vec(value).unwrap_or_default();
    line.push(b'\n');
    axum::body::Bytes::from(line)
}

#[cfg(test)]
mod tests {
    use super::{Scope, StreamParams, spec};

    fn params(scope: &str) -> StreamParams {
        StreamParams {
            q: "invoice".into(),
            provider: "codex".into(),
            project: "/p".into(),
            scope: scope.into(),
            limit: Some(5),
            live: true,
            ..Default::default()
        }
    }

    #[test]
    fn the_viewing_scope_chooses_which_sources_are_asked() {
        assert_eq!(spec(params(""), false).unwrap().scope, Scope::All);
        assert_eq!(spec(params("all"), false).unwrap().scope, Scope::All);
        assert_eq!(
            spec(params("local"), false).unwrap().scope,
            Scope::ThisMachine
        );
        assert_eq!(
            spec(params("node:xenia"), false).unwrap().scope,
            Scope::Machine("xenia".into())
        );
        for bad in ["node:", "everywhere"] {
            assert!(spec(params(bad), false).is_err(), "{bad}");
        }
    }

    #[test]
    fn filters_and_options_reach_the_search() {
        let spec = spec(params("all"), true).unwrap();
        assert_eq!(spec.filters.project, ["/p"]);
        assert!(spec.live && spec.meaning);
        // A dashboard that serves only the shared index has none of its own.
        assert!(!spec.local_index);
        let words_only = super::spec(
            StreamParams {
                words_only: true,
                ..params("all")
            },
            false,
        )
        .unwrap();
        assert!(!words_only.meaning);
        assert!(
            super::spec(
                StreamParams {
                    provider: "nope".into(),
                    ..params("all")
                },
                false
            )
            .is_err()
        );
    }
}

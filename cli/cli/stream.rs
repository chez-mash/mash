//! `mash search --stream`: federated search printed as it happens, one line of
//! JSON per update, for tools such as Raycast that show results as they come.
use super::{Cli, Command, Paths, Response};
use crate::AppError;
use crate::core::search::federated::{self, Scope, Spec};
use futures_util::StreamExt;
use std::io::Write;

pub(super) async fn run(cli: Cli) -> Result<Response, AppError> {
    let Command::Search {
        query,
        limit,
        filters,
        live,
        scope,
        ..
    } = cli.command
    else {
        return Err(AppError::internal("stream runs only for search"));
    };
    let scope = match scope.as_str() {
        "all" => Scope::All,
        "local" => Scope::ThisMachine,
        other => match other.strip_prefix("node:") {
            Some(node) if !node.is_empty() => Scope::Machine(node.to_owned()),
            _ => return Err(AppError::usage("--scope is local, all or node:<machine>")),
        },
    };
    let spec = Spec {
        query,
        limit: limit.unwrap_or(if live { 8 } else { 10 }),
        filters: filters.into_selection()?,
        scope,
        live,
        meaning: true,
        local_index: true,
    };
    let paths = Paths {
        config: cli.config,
        db: cli.db,
        models_dir: cli.models_dir,
    };
    let mut updates = std::pin::pin!(federated::search(paths, spec, None).await?);
    let stdout = std::io::stdout();
    let mut results = 0;
    while let Some(update) = updates.next().await {
        results = update.results.len();
        let mut out = stdout.lock();
        let written = serde_json::to_writer(&mut out, &update).is_ok()
            && out.write_all(b"\n").is_ok()
            && out.flush().is_ok();
        // A reader that has gone (a closed pipe) stops the search: leaving
        // the loop drops the stream and the sources still running.
        if !written {
            break;
        }
    }
    // Every line is already written; nothing more to print.
    Ok(Response::Streamed {
        results: u64::try_from(results).unwrap_or(u64::MAX),
    })
}

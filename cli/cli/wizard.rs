//! Terminal-only setup presentation. All mutations remain in the typed core.
mod prompts;

use super::{AppError, Cli, Operation, Response, core};
use crate::config::{self, QdrantConfig};
use crate::core::InitRequest;
use std::time::Duration;

pub(super) const fn interactive(yes: bool, input_terminal: bool, output_terminal: bool) -> bool {
    !yes && input_terminal && output_terminal
}

pub(super) async fn run(cli: Cli) -> Result<Response, AppError> {
    let request = cli.into_request()?;
    let Operation::Init(init) = request.operation else {
        return Err(AppError::internal("wizard requires init"));
    };
    eprintln!(
        "Mash setup\nMash is open source and runs on your computer. Esc or Ctrl-C cancels before changes."
    );
    let mut customize = false;
    loop {
        let paths = request.paths.clone();
        let init = init.clone();
        let draft = crate::blocking(move || prompts::collect(&paths, init, customize)).await?;
        let Some(mut draft) = draft else {
            return Ok(Response::InitCancelled { cancelled: true });
        };
        let ready = !draft.init.index || services_ready(&draft).await;
        let (decision, init) = crate::blocking(move || {
            let decision = prompts::confirm(&mut draft, ready)?;
            Ok((decision, draft.init))
        })
        .await?;
        match decision {
            None => return Ok(Response::InitCancelled { cancelled: true }),
            Some(Decision::Customize) => customize = true,
            Some(Decision::Apply) => {
                return core::execute(core::Request {
                    paths: request.paths,
                    operation: Operation::Init(init),
                })
                .await
                .map(Box::new)
                .map(Response::Setup);
            }
        }
    }
}

async fn services_ready(draft: &Draft) -> bool {
    let observed = crate::views::qdrant::reachability(&draft.qdrant).await;
    let mut ready = observed.state == crate::views::qdrant::Reachability::Reachable;
    if !ready {
        eprintln!(
            "The local search service (Qdrant) is unavailable. {}\nYou can save settings now and run `mash init` again once it is running.",
            observed.message.unwrap_or_default()
        );
    }
    if let Some(shared) = &draft.shared {
        eprintln!("Checking shared-server authentication (no history uploaded)...");
        let checked =
            tokio::time::timeout(Duration::from_secs(8), crate::shared::freshness(shared)).await;
        if matches!(checked, Ok(Ok(_))) {
            eprintln!(
                "Shared-server authentication succeeded. Clients and server must use matching Mash releases."
            );
        } else {
            ready = false;
            eprintln!(
                "Could not verify the shared connection. Check the server URL, group, machine ID and token environment variable. You can save configuration without indexing."
            );
        }
    }
    ready
}

enum Decision {
    Apply,
    Customize,
}

struct Draft {
    init: InitRequest,
    qdrant: QdrantConfig,
    shared: Option<config::SharedConfig>,
    summary: String,
    quick: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_wizard_only_prompts_with_input_and_stderr_terminals() {
        for yes in [true, false] {
            for input in [true, false] {
                for output in [true, false] {
                    assert_eq!(interactive(yes, input, output), !yes && input && output);
                }
            }
        }
    }
}

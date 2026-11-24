//! `mash dash`: serve the dashboard, or set up its remote access.
use super::{AppError, Command, DashCommand, Paths};

/// Serve the dashboard, or set up its remote access.
pub(super) fn dash(
    runtime: &tokio::runtime::Runtime,
    paths: Paths,
    command: Command,
) -> Result<(), AppError> {
    match command {
        Command::Dash {
            command:
                Some(DashCommand::Remote {
                    port,
                    shared_only,
                    public,
                    code,
                    qr,
                    remove,
                }),
            ..
        } => runtime.block_on(crate::remote::run(
            paths,
            crate::remote::Options {
                port,
                shared_only,
                public,
            },
            remote_action(code, qr, remove),
        )),
        Command::Dash {
            command: None,
            listen,
            allow_host,
            access_token_env,
            shared_only,
            services_config,
            fleet_config,
            fleet_node,
        } => runtime.block_on(crate::dash::run(
            paths,
            crate::dash::Reach {
                listen,
                hosts: allow_host,
                access_token_env,
                shared_only,
            },
            services_config,
            fleet_config,
            fleet_node,
        )),
        _ => Err(AppError::internal("not a dashboard command")),
    }
}

fn remote_action(code: bool, qr: bool, remove: bool) -> crate::remote::Action {
    use crate::remote::Action as RemoteAction;
    if remove {
        RemoteAction::Remove
    } else if code {
        RemoteAction::Code
    } else if qr {
        RemoteAction::Qr
    } else {
        RemoteAction::Setup
    }
}

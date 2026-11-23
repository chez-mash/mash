//! Joining a shared index from a join string during `mash init`: the plan it
//! writes, then the order that keeps a refused join harmless. The string is
//! checked first, then the usual Qdrant preflight runs, and only then is the
//! one-time string redeemed and the token saved, so a failed check or
//! preflight leaves the string unused and nothing written.
use super::{InitPlan, InitRequest};
use crate::AppError;
use crate::core::models::Paths;

/// The plan that connects this machine to a shared index from a join string:
/// the histories already configured or found here, publishing to the server
/// the string names. It saves nothing: the token is saved by [`redeem_join`]
/// once the server has accepted the string.
pub fn join_plan(paths: &Paths, join: &crate::shared::join::Join) -> Result<InitPlan, AppError> {
    let current = crate::config::load(paths.config.as_deref())?;
    let previous = if current.loaded {
        Some(std::fs::read_to_string(&current.path).map_err(AppError::io)?)
    } else {
        None
    };
    // Keep the variable name an existing configuration already uses.
    let token_env = current
        .shared
        .as_ref()
        .map_or("MASH_NODE_TOKEN", |shared| shared.token_env.as_str())
        .to_owned();
    Ok(InitPlan {
        choices: crate::config::init::Choices {
            records: current.records.clone(),
            providers: if current.loaded {
                current.providers.clone()
            } else {
                crate::config::init::detect()?
            },
            since_days: current.since_days,
            shared: Some(crate::config::SharedConfig {
                group_id: join.group_id.clone(),
                node_id: join.node_id.clone(),
                api_url: join.api_url.clone(),
                token_env,
                publication_redaction: crate::redaction::Policy::default(),
                unshared: Vec::new(),
            }),
            hold_publication: false,
        },
        previous,
    })
}

/// Check a join string before anything changes: a refused, expired or used
/// string, or a name whose history another index published, stops `init`
/// here with nothing saved and the string not used up.
async fn check_join(paths: &Paths, join: &crate::shared::join::Join) -> Result<(), AppError> {
    use crate::shared::join;
    let answer = join::check(join).await?;
    if answer.status != crate::shared::server::enrollment::JoinStatus::Ready {
        return Err(join::refusal(answer.status));
    }
    let database = paths
        .db
        .clone()
        .unwrap_or_else(super::super::default_database_path);
    let local = crate::blocking(move || join::local_instance(&database)).await?;
    match join::conflicting_owner(join, answer.published.as_ref(), local.as_deref()) {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

/// Redeem a checked join string and save the token it yields under Mash's own
/// directory, readable only by the user, so no shell profile needs editing.
/// This is the step that uses the string up, so it runs after every check.
async fn redeem_join(join: &crate::shared::join::Join, plan: &InitPlan) -> Result<(), AppError> {
    let token = crate::shared::join::redeem(join)
        .await?
        .map_err(crate::shared::join::refusal)?;
    let token_env = plan
        .choices
        .shared
        .as_ref()
        .map(|shared| shared.token_env.clone())
        .ok_or_else(|| AppError::internal("a join plan names its shared index"))?;
    crate::blocking(move || {
        let tokens = crate::config::saved_tokens()
            .ok_or_else(|| AppError::configuration("cannot find the home directory"))?;
        crate::config::save_token(&tokens, &token_env, &token)
    })
    .await
}

/// The checks `init` runs before changing anything, with a join's in order.
pub(in crate::core) async fn preflight_with_join(
    paths: &Paths,
    init: &InitRequest,
) -> Result<(), AppError> {
    if let Some(join) = &init.join {
        check_join(paths, join).await?;
    }
    super::preflight(paths, init).await?;
    if let (Some(join), Some(plan)) = (&init.join, &init.plan) {
        redeem_join(join, plan).await?;
    }
    Ok(())
}

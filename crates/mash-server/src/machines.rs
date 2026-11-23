//! The operator's commands for machines: add one and print its join string,
//! and retire what one published. Each exits without serving.
use std::path::Path;

use mash_app::shared::server::{Identity, configuration, enrollment, store};
use mash_model::AppError;

use super::{attachments, sync};

/// Delete everything one configured node published, then exit without
/// serving. The node's machine can publish afresh afterwards, from any local
/// index, as if it had never published. `node` is `GROUP/NODE`.
///
/// The service must be stopped first: this takes the same exclusive hold on
/// the history directory that a running server has. A running server
/// deletes a removed machine's history itself when the dashboard asks.
pub async fn retire(path: &Path, node: &str) -> Result<(), AppError> {
    let config = configuration::load(path).await?;
    let members = configuration::members(&config);
    let identity = members
        .iter()
        .find(|identity| format!("{}/{}", identity.group_id, identity.node_id) == node)
        .ok_or_else(|| {
            AppError::usage("--retire-node names no configured node; use GROUP/NODE from [[nodes]]")
        })?;
    let store = store::initialize(&config, &members, false, false, false).await?;
    let catalog = std::sync::Arc::new(std::sync::Mutex::new(sync::open_catalog(
        &config.silver_dir()?.join("sync"),
    )?));
    let retired = retire_node(
        &store,
        &catalog,
        config.attachments_dir.as_deref(),
        identity,
    )
    .await?;
    crate::write_json(&mut std::io::stdout(), &retired)
}

/// Delete one node's history, search data, transfer receipts and
/// attachments. Receipts and attachments go with the history they
/// describe: a receipt left behind could acknowledge an upload that no
/// longer exists.
pub(crate) async fn retire_node(
    store: &store::Store,
    catalog: &sync::CatalogHandle,
    attachments_dir: Option<&Path>,
    identity: &Identity,
) -> Result<store::Retired, AppError> {
    let retired = store.retire(identity).await?;
    let (group, name) = (identity.group_id.clone(), identity.node_id.clone());
    sync::catalog(catalog, move |catalog| catalog.retire(&group, &name)).await?;
    if let Some(root) = attachments_dir {
        let directory = attachments::node_directory(root, &identity.group_id, &identity.node_id);
        match tokio::fs::remove_dir_all(&directory).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(AppError::io(error)),
        }
    }
    Ok(retired)
}

/// Add a node to this server and print the join string for its machine.
/// The server keeps only a hash of the string's one-time secret, in a file
/// beside its configuration, and a running server accepts the machine
/// without a restart. `node` is `GROUP/NODE`, and `api_url` is the address
/// that machine will use.
pub async fn add_node(path: &Path, node: &str, api_url: &str) -> Result<(), AppError> {
    let config = configuration::load(path).await?;
    let (group_id, _) = node_name(node)?;
    let new_group = !configuration::members(&config)
        .iter()
        .any(|member| member.group_id == group_id);
    let join = mint(path, node, api_url).await?;
    crate::write_json(
        &mut std::io::stdout(),
        &serde_json::json!({
            "node": node,
            "join": join,
            "expires_in_hours": enrollment::JOIN_LIFETIME / 3600,
            "next": if new_group {
                "This group is new to the server: restart mash-server so it accepts machines in it. Then, on that machine, run `mash init --join` with the join value, or paste it into Join a shared index in Mash. It works once, for 24 hours, and contains the machine's credential: pass it privately."
            } else {
                "On that machine, run `mash init --join` with the join value, or paste it into Join a shared index in Mash. A running server accepts it without a restart. It works once, for 24 hours, and contains the machine's credential: pass it privately."
            },
        }),
    )
}

/// `GROUP/NODE` as its two validated names.
pub(crate) fn node_name(node: &str) -> Result<(&str, &str), AppError> {
    let (group_id, node_id) = node
        .split_once('/')
        .ok_or_else(|| AppError::usage("--add-node takes GROUP/NODE, such as personal/laptop"))?;
    crate::config::validate_identifier(group_id, "group_id")?;
    crate::config::validate_identifier(node_id, "node_id")?;
    Ok((group_id, node_id))
}

/// Record a new machine by the hash of a one-time secret and return its
/// join string.
pub(crate) async fn mint(path: &Path, node: &str, api_url: &str) -> Result<String, AppError> {
    let config = configuration::load(path).await?;
    let (group_id, node_id) = node_name(node)?;
    let configured = configuration::configured(&config);
    let file = path.to_path_buf();
    let (group, name, address) = (group_id.to_owned(), node_id.to_owned(), api_url.to_owned());
    crate::blocking(move || enrollment::mint(&file, &configured, &group, &name, Some(&address)))
        .await
}

/// Make an admin key and print it once. The server keeps only its hash, and
/// a running server accepts it within a second.
pub fn add_admin(path: &Path, name: &str) -> Result<(), AppError> {
    let key = enrollment::update(path, |added| {
        mash_app::shared::server::admin::create(added, name, enrollment::now())
    })?;
    crate::write_json(
        &mut std::io::stdout(),
        &serde_json::json!({
            "admin_key": name,
            "key": key,
            "next": "In Mash on the machine you manage from, open Settings, then Shared index, then Manage this server, and paste this key. It is shown only now and lets its holder add, remove and replace machines: keep it in a password manager and pass it privately. Run with --revoke-admin to stop accepting it.",
        }),
    )
}

/// Stop accepting an admin key.
pub fn revoke_admin(path: &Path, name: &str) -> Result<(), AppError> {
    enrollment::update(path, |added| {
        mash_app::shared::server::admin::revoke(added, name)
    })?;
    crate::write_json(
        &mut std::io::stdout(),
        &serde_json::json!({"revoked_admin_key": name}),
    )
}

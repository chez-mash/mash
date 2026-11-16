//! Native Claude identities and deterministic reconciliation of source contributions.
use std::collections::{BTreeMap, BTreeSet};

use super::NormalizedMessage;
use super::messages::{Builder, Messages};
use crate::AppError;
use crate::message_context::MessageContext;

pub type Lineage = BTreeMap<String, Option<String>>;

/// Whether `candidate` leaves less of `content` searchable than `kept`. No
/// projection means the whole content. What is searched is derived from the
/// content, and a newer reading derives less of it: a copy another session
/// file contributed before a reread still carries the wider one, so the
/// narrowest is kept. Ties break on the text, never on arrival order.
fn narrower(candidate: Option<&str>, kept: Option<&str>, content: &str) -> bool {
    let size = |projection: Option<&str>| projection.map_or(content.len(), str::len);
    (size(candidate), candidate) < (size(kept), kept)
}

/// Check a further copy of a message already seen and keep what it adds.
/// Copies retained from an earlier interpretation may lack tool activity that
/// a newer copy carries; the activity is kept.
fn merge_copy(
    previous: &NormalizedMessage,
    same_parent: bool,
    message: NormalizedMessage,
    tools: &mut BTreeMap<String, crate::tools::Activity>,
    projections: &mut BTreeMap<String, Option<String>>,
) -> Result<(), AppError> {
    let differing = if previous.role != message.role {
        Some("role")
    } else if previous.content != message.content {
        Some("content")
    } else if previous.created_at != message.created_at {
        Some("time")
    } else if same_parent {
        None
    } else {
        Some("parent")
    };
    if let Some(differing) = differing {
        // Name what differs: the identity is not history, and without it
        // the failure cannot be told apart from a changed reading.
        return Err(AppError::configuration(format!(
            "Claude native message identity has conflicting content or ancestry \
             (message {}: {differing} differs between session files)",
            message.id
        )));
    }
    let kept = projections
        .get(&message.id)
        .unwrap_or(&previous.dialogue_text);
    if narrower(
        message.dialogue_text.as_deref(),
        kept.as_deref(),
        &message.content,
    ) {
        projections.insert(message.id.clone(), message.dialogue_text);
    }
    if previous.tools.is_empty() && !message.tools.is_empty() {
        tools.insert(message.id, message.tools);
    }
    Ok(())
}

/// Which reading of its session file a copy of a message came from.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Reading {
    /// Read the way this release reads history.
    Current,
    /// Read by an earlier release and waiting to be read again. Its roles,
    /// text and ancestry may differ; it yields to any current copy.
    Earlier,
}

struct Node {
    position: usize,
    created_at: Option<i64>,
    parent: Option<String>,
    reading: Reading,
}

/// Reconcile matching native identities and order every retained branch after
/// its parent. Missing ancestors remain roots until another contribution supplies
/// them. Timestamp and identity break ties; source arrival order never does.
pub fn reconcile(
    contributions: impl IntoIterator<Item = Result<(NormalizedMessage, Option<String>), AppError>>,
) -> Result<(Messages, Lineage), AppError> {
    reconcile_readings(contributions.into_iter().map(|contribution| {
        contribution.map(|(message, parent)| (message, parent, Reading::Current))
    }))
}

/// `reconcile`, for copies that may come from files not yet read again. While
/// a reread is under way a message can have a current copy from one session
/// file and an earlier one from another; they are not required to agree.
pub fn reconcile_readings(
    contributions: impl IntoIterator<
        Item = Result<(NormalizedMessage, Option<String>, Reading), AppError>,
    >,
) -> Result<(Messages, Lineage), AppError> {
    let mut payloads = Builder::default();
    let mut nodes = BTreeMap::<String, Node>::new();
    let mut tools = BTreeMap::<String, crate::tools::Activity>::new();
    let mut projections = BTreeMap::<String, Option<String>>::new();
    for contribution in contributions {
        let (message, parent, reading) = contribution?;
        match nodes.get(&message.id) {
            Some(node) if node.reading == reading => {
                let previous = payloads
                    .get(node.position)?
                    .ok_or_else(|| AppError::internal("Claude reconciliation lost a payload"))?;
                let same_parent = node.parent == parent;
                merge_copy(
                    &previous,
                    same_parent,
                    message,
                    &mut tools,
                    &mut projections,
                )?;
            }
            // The current copy is already held.
            Some(_) if reading == Reading::Earlier => {}
            // New, or a current copy replacing an earlier one and all it added.
            _ => {
                let id = message.id.clone();
                tools.remove(&id);
                projections.remove(&id);
                let created_at = message.created_at;
                let position = payloads.push(message)?;
                nodes.insert(
                    id,
                    Node {
                        position,
                        created_at,
                        parent,
                        reading,
                    },
                );
            }
        }
    }
    let mut children = BTreeMap::<String, Vec<String>>::new();
    let mut ready = BTreeSet::new();
    for (id, node) in &nodes {
        if let Some(parent) = node
            .parent
            .as_ref()
            .filter(|parent| nodes.contains_key(*parent))
        {
            children.entry(parent.clone()).or_default().push(id.clone());
        } else {
            ready.insert((node.created_at, id.clone()));
        }
    }
    let mut positions = BTreeMap::new();
    let mut lineage = Lineage::new();
    let mut messages = Builder::default();
    while let Some((_, id)) = ready.pop_first() {
        let node = nodes.remove(&id).ok_or_else(|| {
            AppError::internal("Claude reconciliation selected an unknown message")
        })?;
        let mut message = payloads
            .get(node.position)?
            .ok_or_else(|| AppError::internal("Claude reconciliation lost a payload"))?
            .into_owned();
        let parent = node.parent;
        if let Some(activity) = tools.remove(&id) {
            message.tools = activity;
        }
        if let Some(projection) = projections.remove(&id) {
            message.dialogue_text = projection;
        }
        message.ordinal = i64::try_from(messages.len())
            .map_err(|_| AppError::internal("Claude chat contains too many messages"))?;
        message.context = parent
            .as_ref()
            .filter(|id| positions.contains_key(*id))
            .map_or(MessageContext::Root, |id| MessageContext::Parent {
                id: id.clone(),
            });
        positions.insert(id.clone(), message.ordinal);
        lineage.insert(id.clone(), parent);
        messages.push(message)?;
        for child in children.remove(&id).unwrap_or_default() {
            let node = nodes
                .get(&child)
                .ok_or_else(|| AppError::internal("Claude reconciliation lost a child message"))?;
            ready.insert((node.created_at, child));
        }
    }
    if !nodes.is_empty() {
        return Err(AppError::configuration(
            "Claude message ancestry contains a cycle",
        ));
    }
    Ok((messages.finish()?, lineage))
}

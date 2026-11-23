//! How far each machine's search documents on this server have caught up with
//! what it published: missing, stale or ready, for word search and for search
//! by meaning. It is the one place that decides this, so the freshness that
//! publishing machines read and the status any machine of the group reads
//! cannot disagree. The server records no failure state and no build time for
//! views, so neither is claimed here.
use super::{Identity, Store};
use crate::AppError;
use crate::shared::silver_store::Head;
use crate::views::qdrant::NodeViews;
use crate::views::readiness::{Artifacts, State};
use std::collections::BTreeMap;

/// One machine's search documents measured against its committed head.
pub(super) struct Readiness {
    pub lexical_current: bool,
    pub views_current: bool,
    pub artifacts: Artifacts,
}

impl Store {
    /// `views` as stored for `node`, against the silver revision `head` names.
    pub(super) fn readiness(
        &self,
        node: &Identity,
        views: Option<&NodeViews>,
        head: &Head,
    ) -> Readiness {
        let silver = head.silver_revision.as_ref();
        let lexical_current =
            views.is_some_and(|views| silver.is_some() && views.silver.as_ref() == silver);
        let views_current = views.is_some_and(|views| views.current(&self.views, node, silver));
        let dense = if self.views.build().dense_enabled() {
            State::publication(views.is_some(), views_current)
        } else {
            State::Disabled
        };
        Readiness {
            lexical_current,
            views_current,
            artifacts: Artifacts {
                metadata: State::publication(true, true),
                lexical: State::publication(views.is_some(), lexical_current),
                dense,
            },
        }
    }

    /// Each machine of `identity`'s group that has published, with the state
    /// of its search documents. A machine that never published has no head and
    /// is not listed.
    pub async fn search_readiness(
        &self,
        identity: &Identity,
    ) -> Result<BTreeMap<String, Artifacts>, AppError> {
        let mut machines = BTreeMap::new();
        for (node_id, head) in self.heads(&identity.group_id).await? {
            let node = Identity {
                node_id: node_id.clone(),
                ..identity.clone()
            };
            let views = self.views.node_views(&node).await?;
            machines.insert(
                node_id,
                self.readiness(&node, views.as_ref(), &head).artifacts,
            );
        }
        Ok(machines)
    }
}

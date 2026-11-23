//! Group search: which nodes a query may read, and hydrating their hits from
//! silver. A node whose dense views are not yet certified, such as one in the
//! middle of its first publication, fails a related-discussion search only
//! when the query could read its documents.
use super::{Identity, Store, fan_out};
use crate::AppError;
use crate::shared::protocol;
use crate::shared::silver_store;
use crate::views::qdrant as qdrant_index;
use std::collections::{BTreeMap, BTreeSet};

/// The nodes a search reads, and whether any node has views at all.
struct Searchable {
    nodes: BTreeSet<String>,
    initialized: bool,
}

impl Store {
    /// Searchable nodes. Lexical documents survive recipe and model changes;
    /// related-discussion search fails closed while any searched node's dense
    /// views are incompatible, instead of silently omitting that node. A node
    /// the query's filter rules out is not searched, so its views cannot make
    /// the search stale: another machine's publication must not interrupt a
    /// search of this one.
    async fn searchable_nodes(
        &self,
        identity: &Identity,
        input: &protocol::Search,
        dense: bool,
    ) -> Result<Searchable, AppError> {
        // Replay can create empty views for configured nodes that have never
        // published. Only committed publication heads establish searchable
        // membership; Qdrant metadata alone must not add a node to the group.
        let nodes = self
            .heads(&identity.group_id)
            .await?
            .into_keys()
            .filter(|node_id| !(input.exclude_self && node_id == &identity.node_id))
            .collect::<Vec<_>>();
        let checked = fan_out(nodes, |node_id| {
            let node = Identity {
                node_id: node_id.clone(),
                ..identity.clone()
            };
            let searched = input
                .filter
                .as_ref()
                .is_none_or(|filter| filter.admits_node(&node_id));
            async move {
                let Some(views) = self.views.node_views(&node).await? else {
                    return Ok(None);
                };
                if dense && searched && !views.searchable(&self.views, &node) {
                    return Err(AppError::shared(
                        "shared views are stale, incomplete or incompatible; republish from nodes or restart the API with --replay-views",
                    ));
                }
                Ok(Some((node_id, searched)))
            }
        })
        .await?;
        let initialized = checked.iter().any(Option::is_some);
        Ok(Searchable {
            nodes: checked
                .into_iter()
                .flatten()
                .filter_map(|(node_id, searched)| searched.then_some(node_id))
                .collect(),
            initialized,
        })
    }

    pub async fn search(
        &self,
        identity: &Identity,
        input: protocol::Search,
    ) -> Result<protocol::Candidates, AppError> {
        qdrant_index::validate_search(&input, identity)?;
        self.views.views_configuration(identity).await?;
        let exact = !input.intent.semantic();
        let searchable = self.searchable_nodes(identity, &input, !exact).await?;
        if !searchable.initialized && exact && !input.exclude_self {
            return Err(AppError::shared(
                "shared lexical index has not been initialized; run mash index",
            ));
        }
        let mut candidates = self
            .views
            .search(identity, input, &searchable.nodes)
            .await?;
        self.hydrate(identity, &mut candidates).await?;
        Ok(candidates)
    }

    /// Derived turns and chat facts live only in silver and never trigger
    /// point rewrites.
    async fn hydrate(
        &self,
        identity: &Identity,
        candidates: &mut protocol::Candidates,
    ) -> Result<(), AppError> {
        use crate::storage::{chat_facts, turns};
        let mut wanted = BTreeMap::<String, BTreeSet<_>>::new();
        for hit in candidates.lexical.iter().chain(&candidates.semantic) {
            if let Some(node) = &hit.node_id {
                wanted.entry(node.clone()).or_default().insert((
                    hit.id.clone(),
                    hit.provider,
                    hit.chat_id.clone(),
                ));
            }
        }
        let found = fan_out(wanted, |(node, messages)| async move {
            let scope = silver_store::node_identity(&identity.group_id, &node);
            let rows = self
                .read(&scope, move |connection| {
                    let mut chats = BTreeMap::new();
                    messages
                        .into_iter()
                        .map(|(id, provider, chat)| {
                            let turn = turns::message_turn(connection, provider, &chat, &id)?;
                            if !chats.contains_key(&chat) {
                                let facts = chat_facts::read(connection, provider, &chat)?;
                                chats.insert(chat.clone(), facts);
                            }
                            Ok((id, (turn, chats.get(&chat).cloned().flatten())))
                        })
                        .collect::<Result<BTreeMap<_, _>, AppError>>()
                })
                .await?;
            Ok::<_, AppError>((node, rows))
        })
        .await?
        .into_iter()
        .flat_map(|(node, rows)| {
            rows.into_iter()
                .map(move |(id, row)| ((node.clone(), id), row))
        })
        .collect::<BTreeMap<_, _>>();
        for hit in candidates
            .lexical
            .iter_mut()
            .chain(&mut candidates.semantic)
        {
            if let Some(node) = &hit.node_id {
                let (turn, facts) = found
                    .get(&(node.clone(), hit.id.clone()))
                    .cloned()
                    .unwrap_or_default();
                hit.turn_id = turn;
                if let Some(facts) = facts {
                    facts.apply(hit);
                }
            }
        }
        Ok(())
    }
}

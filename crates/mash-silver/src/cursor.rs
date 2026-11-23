//! Native Cursor Agent and IDE histories, selected by their physical schemas.
use super::{Chat, HistorySource, NormalizedMessage, ParsedFile, SourceIdentity, records};
use crate::{AppError, message_context::MessageContext, provider::Provider, role::Role};
use std::{collections::HashMap, path::Path};

mod evidence;
pub mod ide;
pub mod models;
pub mod proto;
mod store;
mod text;
use store::Store;

fn invalid(message: &str) -> AppError {
    AppError::configuration(format!("Cursor Agent: {message}"))
}

pub fn discover(path: &Path) -> Result<Vec<HistorySource>, AppError> {
    match format(path)? {
        Format::Ide => return ide::discover(path),
        Format::WorkspaceState => return Ok(Vec::new()),
        Format::Agent => {}
    }
    let store = Store::open(path)?;
    let metadata = store.metadata()?;
    Ok(vec![HistorySource {
        content: None,
        identity: SourceIdentity {
            path: path.into(),
            session: Some(metadata.agent_id),
        },
        activity_ns: None,
        estimated_bytes: None,
        dependencies: Vec::new(),
    }])
}

#[derive(Default)]
struct Messages {
    items: Vec<NormalizedMessage>,
    occurrences: HashMap<String, u64>,
    bytes: usize,
    /// The store records its turns. Without them nothing says which user
    /// messages were typed, so the text itself decides.
    turns_recorded: bool,
    /// The folder the agent ran in, as its injected context states it.
    working_directory: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Origin {
    Archive,
    Current,
}

impl Messages {
    fn push(
        &mut self,
        store: &mut Store,
        id: &[u8],
        session: &str,
        origin: Origin,
        child: bool,
        recorded: &mut evidence::Recorded,
    ) -> Result<(), AppError> {
        let message: models::Message = records::decode(&store.blob(id)?)?;
        if self.working_directory.is_none() && message.role == Role::User {
            self.working_directory = text::working_directory(&message);
        }
        let evidence = if origin == Origin::Archive {
            None
        } else {
            recorded
                .get_mut(&evidence::key(&message.role, &text::spoken(&message)))
                .and_then(std::collections::VecDeque::pop_front)
        };
        let hidden = child
            || message.summary()
            || evidence.is_some_and(|e| !e.visible)
            || (self.turns_recorded
                && origin == Origin::Current
                && message.role == Role::User
                && evidence.is_none());
        for entry in text::entries(&message) {
            let mut text = entry.text;
            if hidden {
                text.projection.clear();
            }
            if text.canonical.trim().is_empty() {
                continue;
            }
            self.bytes = self
                .bytes
                .saturating_add(text.canonical.capacity())
                .saturating_add(text.projection.capacity())
                .saturating_add(entry.tools.retained_size())
                .saturating_add(512);
            if self.bytes > 32 * 1024 * 1024 || self.items.len() == 65_536 {
                return Err(invalid(
                    "normalized history exceeds the retained replay limit",
                ));
            }
            let fingerprint =
                super::parsers::stable_id("turn", &[entry.role.as_str(), &text.canonical]);
            let occurrence = self.occurrences.entry(fingerprint.clone()).or_default();
            let message_id = super::parsers::stable_id(
                "message",
                &[
                    Provider::Cursor.as_str(),
                    session,
                    &fingerprint,
                    &occurrence.to_string(),
                ],
            );
            *occurrence += 1;
            self.items.push(NormalizedMessage {
                attachments: Vec::new(),
                id: message_id,
                ordinal: i64::try_from(self.items.len())
                    .map_err(|_| invalid("too many messages"))?,
                context: MessageContext::Linear,
                role: entry.role,
                content: text.canonical,
                dialogue_text: Some(text.projection),
                created_at: evidence.and_then(|e| e.timestamp),
                tools: entry.tools,
            });
        }
        Ok(())
    }
}

/// The agent's identity within this machine. The same agent ID can exist
/// under two workspace folders with different histories, so the folder the
/// store sits in is part of it.
fn session_scope(path: &Path, agent_id: &str) -> String {
    path.parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .and_then(std::ffi::OsStr::to_str)
        .map_or_else(
            || agent_id.to_owned(),
            |folder| format!("{folder}/{agent_id}"),
        )
}

/// Cursor names the agent that started a subagent. Its parent lives in the
/// same workspace folder, so it has the identity `parse_agent` gives it there.
fn cursor_subagent(path: &Path, child: &models::Subagent, name: &str) -> super::subagent::Subagent {
    let scope = session_scope(path, child.parent_agent_id.trim());
    super::subagent::Subagent {
        parent_chat_id: Some(super::parsers::stable_id(
            "session",
            &[Provider::Cursor.as_str(), &scope],
        )),
        label: super::subagent::label(Some(name)),
        role: super::subagent::label(Some(&child.type_name)),
        depth: None,
        finished: None,
        spawn_call_id: Some(child.tool_call_id.trim().to_owned()),
    }
}

/// When the store was last written, for chats whose messages carry no times.
fn modified_ms(path: &Path) -> Option<i64> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let elapsed = modified.duration_since(std::time::UNIX_EPOCH).ok()?;
    i64::try_from(elapsed.as_millis()).ok()
}

pub fn parse(source: &SourceIdentity) -> Result<ParsedFile, AppError> {
    match format(&source.path)? {
        Format::Ide => ide::parse(source),
        Format::Agent => parse_agent(source),
        Format::WorkspaceState => Err(invalid("workspace state holds no chat history")),
    }
}

#[derive(PartialEq, Eq)]
pub enum Format {
    Agent,
    Ide,
    /// An editor workspace's ordinary state database. Cursor keeps one per
    /// opened folder, and most hold settings only, with no chat table.
    WorkspaceState,
}

fn format(path: &Path) -> Result<Format, AppError> {
    let connection = super::sqlite_source::open(path)?;
    let mut statement = connection
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name IN ('meta','blobs','ItemTable','cursorDiskKV')")
        .map_err(AppError::database)?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(AppError::database)?
        .collect::<Result<std::collections::BTreeSet<_>, _>>()
        .map_err(AppError::database)?;
    match (
        names.contains("meta") && names.contains("blobs"),
        names.contains("ItemTable") && names.contains("cursorDiskKV"),
    ) {
        (true, false) => Ok(Format::Agent),
        (false, true) => Ok(Format::Ide),
        (false, false) if names.len() == 1 && names.contains("ItemTable") => {
            Ok(Format::WorkspaceState)
        }
        _ => Err(AppError::configuration(
            "unsupported or ambiguous Cursor storage schema",
        )),
    }
}

fn parse_agent(source: &SourceIdentity) -> Result<ParsedFile, AppError> {
    let mut store = Store::open(&source.path)?;
    let metadata = store.metadata()?;
    if source.session.as_deref() != Some(metadata.agent_id.as_str()) {
        return Err(invalid("source and native session identities disagree"));
    }
    let id =
        hex::decode(&metadata.latest_root_blob_id).map_err(|_| invalid("invalid root identity"))?;
    let root = if id.is_empty() {
        proto::Root::default()
    } else {
        store.protobuf::<proto::Root>(&id, &[21])?
    };
    if !root.turns.is_empty() && root.messages.is_empty() && root.archives.is_empty() {
        return Err(invalid("native turns have no reachable transcript history"));
    }
    let mut recorded = evidence::current(&mut store, &root)?;
    let scope = session_scope(&source.path, &metadata.agent_id);
    let mut messages = Messages {
        turns_recorded: !root.turns.is_empty(),
        ..Messages::default()
    };
    for id in &root.archives {
        let archive: proto::Archive = store.protobuf(id, &[])?;
        for id in archive.messages {
            messages.push(
                &mut store,
                &id,
                &scope,
                Origin::Archive,
                metadata.subagent_info.is_some(),
                &mut recorded,
            )?;
        }
    }
    for id in root.messages {
        messages.push(
            &mut store,
            &id,
            &scope,
            Origin::Current,
            metadata.subagent_info.is_some(),
            &mut recorded,
        )?;
    }
    let created_at = evidence::timestamp(root.started_at)?.or(Some(metadata.created_at));
    // Stores without recorded turns keep no message times; the store's own
    // last write is then the best account of when the chat last moved.
    let updated_at = messages
        .items
        .iter()
        .filter_map(|m| m.created_at)
        .max()
        .or_else(|| modified_ms(&source.path).max(created_at))
        .or(created_at);
    let project = unique(
        root.workspaces
            .into_iter()
            .filter_map(|uri| crate::platform::file_uri_path(&uri)),
    )
    .or_else(|| messages.working_directory.take());
    let repository = unique(
        root.repositories
            .into_iter()
            .map(|r| r.path)
            .filter(|p| !p.trim().is_empty()),
    );
    let title = crate::dialogue::title(&metadata.name)
        .filter(|n| n != "New Agent")
        .or_else(|| {
            messages
                .items
                .iter()
                .filter(|m| m.role == Role::User)
                .find_map(|m| {
                    crate::dialogue::title(m.dialogue_text.as_deref().unwrap_or_default())
                })
        });
    Ok(ParsedFile {
        chat: Some(Chat {
            id: super::parsers::stable_id("session", &[Provider::Cursor.as_str(), &scope]),
            provider: Provider::Cursor,
            title_origin: None,
            timestamp_origins: None,
            claude_lineage: None,
            origin: super::subagent::Origin::of(
                metadata
                    .subagent_info
                    .as_ref()
                    .map(|child| cursor_subagent(&source.path, child, &metadata.name)),
            ),
            source: source.clone(),
            project,
            repository,
            title,
            created_at,
            updated_at,
            messages: super::messages::Messages::stage(messages.items)?,
        }),
        malformed_records: 0,
        first_record_issue: None,
    })
}

/// A tool name or call identity as a single-line label. Cursor joins two
/// provider identities with a newline; published activity allows no control
/// characters, so each becomes a space, the same way on a call and its result.
fn label(value: Option<&str>) -> Option<String> {
    let label: String = value?
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let label = label.trim();
    (!label.is_empty() && label.len() <= 4096).then(|| label.to_owned())
}

fn unique(values: impl Iterator<Item = String>) -> Option<String> {
    let values = values.collect::<std::collections::BTreeSet<_>>();
    (values.len() == 1)
        .then(|| values.into_iter().next())
        .flatten()
}

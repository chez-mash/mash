//! `OpenCode`'s native session/project database and its two upstream message layouts.
use super::{Chat, HistorySource, NormalizedMessage, ParsedFile, SourceIdentity, sqlite_source};
use crate::{AppError, provider::Provider, role::Role};
use rusqlite::{Connection, OptionalExtension};
use serde::Deserialize;
use std::path::Path;

mod limits;
use limits::Budget;

fn has_table(connection: &Connection, table: &str) -> Result<bool, AppError> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name=?1)",
            [table],
            |row| row.get(0),
        )
        .map_err(AppError::database)
}

pub fn discover(path: &Path) -> Result<Vec<HistorySource>, AppError> {
    let connection = sqlite_source::open(path)?;
    let tables = session_tables(&connection)?;
    if tables.is_empty() {
        return Ok(Vec::new());
    }
    // Upstream moved sessions to `session_v2`; both layouts can coexist in one
    // database while older sessions are still being read.
    let query = format!(
        "SELECT id, MAX(time_updated) AS time_updated FROM ({}) GROUP BY id ORDER BY id",
        tables
            .iter()
            .map(|table| format!("SELECT id,time_updated FROM {table}"))
            .collect::<Vec<_>>()
            .join(" UNION ALL ")
    );
    let mut statement = connection.prepare(&query).map_err(AppError::database)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>("id")?,
                row.get::<_, i64>("time_updated")?,
            ))
        })
        .map_err(AppError::database)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::database)?;
    rows.into_iter().map(|(session, activity)| {
        if session.trim().is_empty() { return Err(AppError::configuration("OpenCode session identity is empty")); }
        let activity_ns = activity.checked_mul(1_000_000).filter(|n| *n >= 0)
            .ok_or_else(|| AppError::configuration("OpenCode session activity is out of range"))?;
        let bytes: i64 = if uses_current(&connection, &session)? {
            connection.query_row("SELECT COALESCE(SUM(length(CAST(data AS BLOB))),0) FROM session_message WHERE session_id=?1",
                [&session], |row| row.get(0)).map_err(AppError::database)?
        } else {
            connection.query_row("SELECT (SELECT COALESCE(SUM(length(CAST(data AS BLOB))),0) FROM message WHERE session_id=?1)
                +(SELECT COALESCE(SUM(length(CAST(data AS BLOB))),0) FROM part WHERE session_id=?1)",
                [&session], |row| row.get(0)).map_err(AppError::database)?
        };
        Ok(HistorySource { content: None, identity: SourceIdentity {path:path.to_path_buf(),session:Some(session)},
            activity_ns:Some(activity_ns), dependencies:Vec::new(), estimated_bytes:Some(u64::try_from(bytes)
                .map_err(|_| AppError::configuration("invalid OpenCode session byte count"))?) })
    }).collect()
}

/// Session tables present in this database, newest layout first.
fn session_tables(connection: &Connection) -> Result<Vec<&'static str>, AppError> {
    let mut tables = Vec::new();
    for table in ["session_v2", "session"] {
        if has_table(connection, table)? {
            tables.push(table);
        }
    }
    Ok(tables)
}

/// Table holding this session, preferring the current layout.
fn session_table(connection: &Connection, session: &str) -> Result<Option<&'static str>, AppError> {
    for table in session_tables(connection)? {
        let exists: bool = connection
            .query_row(
                &format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id=?1)"),
                [session],
                |row| row.get(0),
            )
            .map_err(AppError::database)?;
        if exists {
            return Ok(Some(table));
        }
    }
    Ok(None)
}

fn uses_current(connection: &Connection, session: &str) -> Result<bool, AppError> {
    if !has_table(connection, "session_message")? {
        return Ok(false);
    }
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM session_message WHERE session_id=?1)",
            [session],
            |row| row.get(0),
        )
        .map_err(AppError::database)
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Kind {
    User,
    Assistant,
    Synthetic,
    System,
    Shell,
    Compaction,
    AgentSwitched,
    ModelSwitched,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize)]
struct Text {
    text: String,
}
#[derive(Deserialize)]
struct User {
    text: String,
    #[serde(default)]
    files: Vec<Attachment>,
}
#[derive(Deserialize)]
struct Attachment {
    /// External files carry a URI; inline images carry base64 data instead and
    /// are not addressable, so they are omitted from the projection.
    #[serde(default)]
    uri: Option<String>,
    name: Option<String>,
    description: Option<String>,
}
#[derive(Deserialize)]
struct Assistant {
    content: Vec<Part>,
}
#[derive(Deserialize)]
struct Shell {
    command: String,
    output: String,
}
#[derive(Deserialize)]
struct Compaction {
    summary: String,
    /// Older databases carry a `recent` excerpt; current ones omit it.
    #[serde(default)]
    recent: Option<String>,
}
#[derive(Deserialize)]
struct LegacyMessage {
    role: Role,
}

struct Session {
    title: String,
    project: String,
    repository: Option<String>,
    created_at: i64,
    updated_at: i64,
}

// Both layouts use typed text, reasoning and tool parts. Tool arguments and unknown
// result shapes stay opaque provider-owned JSON; none of it enters the search projection.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum Part {
    Text {
        text: String,
        #[serde(default)]
        synthetic: bool,
        #[serde(default)]
        ignored: bool,
    },
    Reasoning {
        text: String,
    },
    Tool {
        /// The call's identity in the current layout.
        #[serde(default)]
        id: Option<String>,
        /// The call's identity in the legacy layout.
        #[serde(default, rename = "callID")]
        call_id: Option<String>,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        tool: Option<String>,
        state: ToolState,
    },
    File {
        #[serde(default)]
        filename: Option<String>,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        uri: Option<String>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ToolStatus {
    Completed,
    Error,
    /// Pending, streaming, running and any later state: no result yet.
    #[default]
    #[serde(other)]
    Unsettled,
}

#[derive(Deserialize)]
struct ToolState {
    #[serde(default)]
    status: ToolStatus,
    /// An object once the call runs; a partial string while it streams.
    #[serde(default)]
    input: Option<serde_json::Value>,
    /// The legacy layout's result text.
    #[serde(default)]
    output: Option<Loose>,
    /// The current layout's result.
    #[serde(default)]
    content: Vec<ToolContent>,
    #[serde(default)]
    error: Option<ToolError>,
}

/// Text where upstream documents text, kept as JSON when it is something else.
#[derive(Deserialize)]
#[serde(untagged)]
enum Loose {
    Text(String),
    Other(serde_json::Value),
}

impl Loose {
    fn text(self) -> String {
        match self {
            Self::Text(text) => text,
            Self::Other(value) => value.to_string(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum ToolContent {
    Text {
        text: String,
    },
    File {
        uri: String,
        #[serde(default)]
        name: Option<String>,
    },
    #[serde(other)]
    Unknown,
}

/// The legacy layout records a message; the current one a structured error.
#[derive(Deserialize)]
#[serde(untagged)]
enum ToolError {
    Structured { message: String },
    Other(Loose),
}

impl ToolState {
    /// The result's text and whether it failed, once the call has settled.
    /// A state without a known status still counts when it carries a result.
    fn result(self) -> Option<(String, Option<bool>)> {
        let mut lines = Vec::new();
        lines.extend(self.output.map(Loose::text));
        for content in self.content {
            match content {
                ToolContent::Text { text } => lines.push(text),
                ToolContent::File { uri, name } => {
                    lines.push(format!("File {}: {uri}", name.unwrap_or_default()));
                }
                ToolContent::Unknown => {}
            }
        }
        lines.extend(self.error.map(|error| match error {
            ToolError::Structured { message } => message,
            ToolError::Other(other) => other.text(),
        }));
        let is_error = match self.status {
            ToolStatus::Completed => Some(false),
            ToolStatus::Error => Some(true),
            ToolStatus::Unsettled if lines.is_empty() => return None,
            ToolStatus::Unsettled => None,
        };
        Some((lines.join("\n"), is_error))
    }
}

#[derive(Default)]
struct Content {
    attachments: Vec<crate::attachments::Source>,
    canonical: Vec<String>,
    prose: Vec<String>,
    retained: usize,
}
impl Content {
    fn append(&mut self, canonical: String, prose: Option<String>) -> Result<(), AppError> {
        let bytes = canonical.len()
            + std::mem::size_of::<String>()
            + prose
                .as_ref()
                .map_or(0, |text| text.len() + std::mem::size_of::<String>());
        let retained = self
            .retained
            .checked_add(bytes)
            .ok_or_else(limits::invalid)?;
        if retained > limits::MAX_RETAINED_BYTES {
            return Err(limits::invalid());
        }
        self.retained = retained;
        self.canonical.push(canonical);
        self.prose.extend(prose);
        Ok(())
    }

    fn text(text: String, searchable: bool) -> Result<Self, AppError> {
        let mut content = Self::default();
        let prose = searchable.then(|| text.clone());
        content.append(text, prose)?;
        Ok(content)
    }
}

/// One stored message. A provider message holding reasoning, text and tool
/// parts becomes several, in the order of its parts.
struct Piece {
    /// Distinguishes this piece within its provider message. The first run of
    /// text has none, so it keeps the provider message's own identity.
    suffix: Option<String>,
    role: Role,
    content: Content,
    tools: crate::tools::Activity,
}

impl Piece {
    fn whole(role: Role, content: Content) -> Self {
        Self {
            suffix: None,
            role,
            content,
            tools: crate::tools::Activity::default(),
        }
    }
}

/// Splits a provider message's parts into stored messages.
struct Pieces {
    role: Role,
    /// Messages already complete, in order.
    settled: Vec<Piece>,
    /// The run of text and file parts being collected, with its first part's key.
    run: Option<(String, Content)>,
    /// Text `OpenCode` injected into a prompt, kept apart from what was typed.
    injected: Content,
    /// Whether a run has already taken the provider message's identity.
    named: bool,
}

impl Pieces {
    fn new(role: Role) -> Self {
        Self {
            role,
            settled: Vec::new(),
            run: None,
            injected: Content::default(),
            named: false,
        }
    }

    fn run(&mut self, key: &str) -> &mut Content {
        &mut self
            .run
            .get_or_insert_with(|| (key.to_owned(), Content::default()))
            .1
    }

    fn flush(&mut self) {
        if let Some((key, content)) = self.run.take() {
            let suffix = std::mem::replace(&mut self.named, true).then(|| format!("text:{key}"));
            self.settled.push(Piece {
                suffix,
                ..Piece::whole(self.role.clone(), content)
            });
        }
    }

    /// `key` identifies the part within its message and stays the same when
    /// the message is read again: the part's row in the legacy layout, its
    /// position in the current one.
    fn part(&mut self, key: &str, part: Part) -> Result<(), AppError> {
        match part {
            Part::Text {
                text,
                synthetic,
                ignored,
            } => {
                if (synthetic || ignored) && self.role == Role::User {
                    self.injected.append(text, None)
                } else {
                    let prose = (!synthetic && !ignored).then(|| text.clone());
                    self.run(key).append(text, prose)
                }
            }
            Part::Reasoning { text } => {
                self.flush();
                if !text.trim().is_empty() {
                    self.settled.push(Piece {
                        suffix: Some(format!("reasoning:{key}")),
                        ..Piece::whole(
                            Role::Assistant,
                            Content::text(format!("[Thinking] {text}"), false)?,
                        )
                    });
                }
                Ok(())
            }
            Part::Tool {
                id,
                call_id,
                name,
                tool,
                state,
            } => {
                self.flush();
                let call_id = call_id.or(id);
                let name = name.or(tool);
                let label = name.as_deref().unwrap_or_default();
                let text = state.input.as_ref().map_or_else(
                    || format!("Tool: {label}"),
                    |input| format!("Tool: {label}\n{input}"),
                );
                self.settled.push(Piece {
                    suffix: Some(format!("tool:{key}")),
                    role: Role::Assistant,
                    content: Content::text(text, false)?,
                    tools: crate::tools::Activity::call(
                        name.as_deref(),
                        call_id.as_deref(),
                        state.input.as_ref(),
                    ),
                });
                if let Some((text, is_error)) = state.result() {
                    self.settled.push(Piece {
                        suffix: Some(format!("result:{key}")),
                        role: Role::Tool,
                        content: Content::text(text, false)?,
                        tools: crate::tools::Activity::result(call_id.as_deref(), is_error),
                    });
                }
                Ok(())
            }
            Part::File {
                filename,
                name,
                url,
                uri,
            } => {
                let content = self.run(key);
                content.attachments.push(crate::attachments::url(
                    filename.as_deref().or(name.as_deref()).unwrap_or("file"),
                    url.as_deref().or(uri.as_deref()).unwrap_or(""),
                ));
                content.append(
                    format!(
                        "File {}: {}",
                        filename.or(name).unwrap_or_default(),
                        url.or(uri).unwrap_or_default()
                    ),
                    None,
                )
            }
            Part::Unknown => Ok(()),
        }
    }

    fn finish(mut self) -> Vec<Piece> {
        self.flush();
        if !self.injected.canonical.is_empty() {
            self.settled.push(Piece {
                suffix: Some("synthetic".to_owned()),
                ..Piece::whole(Role::System, self.injected)
            });
        }
        self.settled
    }
}

fn normalize(
    source: &SourceIdentity,
    id: &str,
    created: i64,
    piece: Piece,
    messages: &mut Vec<NormalizedMessage>,
    budget: &mut Budget,
) -> Result<(), AppError> {
    let Piece {
        suffix,
        role,
        content,
        tools,
    } = piece;
    if created < 0 {
        return Err(AppError::configuration(
            "invalid OpenCode message timestamp",
        ));
    }
    let canonical = content.canonical.join("\n");
    if canonical.trim().is_empty() {
        return Ok(());
    }
    let key = source.key();
    let mut identity = vec![Provider::Opencode.as_str(), &key, id];
    identity.extend(suffix.as_deref());
    let message = NormalizedMessage {
        attachments: content.attachments,
        id: super::parsers::stable_id("message", &identity),
        ordinal: i64::try_from(messages.len())
            .map_err(|_| AppError::internal("too many OpenCode messages"))?,
        context: crate::message_context::MessageContext::Linear,
        dialogue_text: Some(
            crate::dialogue::visible(&role, &content.prose.join("\n")).into_owned(),
        ),
        role,
        content: canonical,
        created_at: Some(created),
        tools,
    };
    // A call's arguments are held twice: as text and as structured input.
    budget.retain(
        std::mem::size_of::<NormalizedMessage>()
            + message.id.len()
            + message.content.len()
            + if message.tools.tool_calls.is_empty() {
                0
            } else {
                message.content.len()
            }
            + message.dialogue_text.as_ref().map_or(0, String::len),
    )?;
    messages.push(message);
    Ok(())
}

fn current(
    connection: &Connection,
    source: &SourceIdentity,
    session: &str,
    budget: &mut Budget,
) -> Result<Vec<NormalizedMessage>, AppError> {
    let mut statement = connection
        .prepare(
            "SELECT id,type,time_created,data,length(CAST(data AS BLOB)) AS bytes,
             length(CAST(data AS BLOB))+length(CAST(id AS BLOB))+length(CAST(type AS BLOB)) AS input_bytes
        FROM session_message WHERE session_id=?1 ORDER BY seq,id",
        )
        .map_err(AppError::database)?;
    let mut rows = statement.query([session]).map_err(AppError::database)?;
    let mut messages = Vec::new();
    while let Some(row) = rows.next().map_err(AppError::database)? {
        budget.read(row.get("input_bytes").map_err(AppError::database)?)?;
        let kind: String = row.get("type").map_err(AppError::database)?;
        let kind = Kind::deserialize(
            serde::de::value::StrDeserializer::<serde::de::value::Error>::new(&kind),
        )
        .map_err(|_| AppError::configuration("invalid OpenCode message type"))?;
        let pieces = match kind {
            Kind::User => {
                let user: User = sqlite_source::bounded_json(row)?;
                let mut content = Content::text(user.text, true)?;
                for file in user.files {
                    let Some(uri) = file.uri else {
                        continue;
                    };
                    content.attachments.push(crate::attachments::url(
                        file.name.as_deref().unwrap_or("file"),
                        &uri,
                    ));
                    content.append(
                        format!(
                            "File {}: {}\n{}",
                            file.name.unwrap_or_default(),
                            uri,
                            file.description.unwrap_or_default()
                        ),
                        None,
                    )?;
                }
                vec![Piece::whole(Role::User, content)]
            }
            Kind::Assistant => {
                let mut pieces = Pieces::new(Role::Assistant);
                let parts = sqlite_source::bounded_json::<Assistant>(row)?.content;
                for (index, part) in parts.into_iter().enumerate() {
                    pieces.part(&index.to_string(), part)?;
                }
                pieces.finish()
            }
            // Synthetic messages are written by OpenCode, never by the person:
            // context beside the system's own, not a prompt.
            Kind::Synthetic | Kind::System => vec![Piece::whole(
                Role::System,
                Content::text(sqlite_source::bounded_json::<Text>(row)?.text, false)?,
            )],
            Kind::Shell => {
                let shell = sqlite_source::bounded_json::<Shell>(row)?;
                vec![Piece::whole(
                    Role::Tool,
                    Content::text(
                        format!("Command: {}\n{}", shell.command, shell.output),
                        false,
                    )?,
                )]
            }
            Kind::Compaction => {
                let compaction = sqlite_source::bounded_json::<Compaction>(row)?;
                let mut text = format!("Compaction: {}", compaction.summary);
                if let Some(recent) = compaction.recent {
                    text.push('\n');
                    text.push_str(&recent);
                }
                vec![Piece::whole(Role::System, Content::text(text, false)?)]
            }
            Kind::AgentSwitched | Kind::ModelSwitched | Kind::Unknown => continue,
        };
        let id: String = row.get("id").map_err(AppError::database)?;
        let created = row.get("time_created").map_err(AppError::database)?;
        for piece in pieces {
            normalize(source, &id, created, piece, &mut messages, budget)?;
        }
    }
    Ok(messages)
}

fn legacy(
    connection: &Connection,
    source: &SourceIdentity,
    session: &str,
    budget: &mut Budget,
) -> Result<Vec<NormalizedMessage>, AppError> {
    let mut statement = connection
        .prepare(
            "SELECT id,time_created,data,length(CAST(data AS BLOB)) AS bytes,
             length(CAST(data AS BLOB))+length(CAST(id AS BLOB)) AS input_bytes
        FROM message WHERE session_id=?1 ORDER BY time_created,id",
        )
        .map_err(AppError::database)?;
    let mut parts = connection
        .prepare(
            "SELECT id,data,length(CAST(data AS BLOB)) AS bytes FROM part
        WHERE message_id=?1 AND session_id=?2 ORDER BY id",
        )
        .map_err(AppError::database)?;
    let mut rows = statement.query([session]).map_err(AppError::database)?;
    let mut messages = Vec::new();
    while let Some(row) = rows.next().map_err(AppError::database)? {
        budget.read(row.get("input_bytes").map_err(AppError::database)?)?;
        let id: String = row.get("id").map_err(AppError::database)?;
        let message: LegacyMessage = sqlite_source::bounded_json(row)?;
        let mut pieces = Pieces::new(message.role);
        let mut fragments = parts
            .query([id.as_str(), session])
            .map_err(AppError::database)?;
        while let Some(fragment) = fragments.next().map_err(AppError::database)? {
            budget.read(fragment.get("bytes").map_err(AppError::database)?)?;
            let key: String = fragment.get("id").map_err(AppError::database)?;
            pieces.part(&key, sqlite_source::bounded_json(fragment)?)?;
        }
        let created = row.get("time_created").map_err(AppError::database)?;
        for piece in pieces.finish() {
            normalize(source, &id, created, piece, &mut messages, budget)?;
        }
    }
    Ok(messages)
}

pub fn parse(source: &SourceIdentity) -> Result<ParsedFile, AppError> {
    let session = source
        .session
        .as_deref()
        .ok_or_else(|| AppError::configuration("OpenCode requires a session identity"))?;
    let connection = sqlite_source::open(&source.path)?;
    let table = session_table(&connection, session)?
        .ok_or_else(|| AppError::configuration("OpenCode session disappeared; retry index"))?;
    let mut budget = Budget::default();
    let metadata_bytes: i64 = connection
        .query_row(
            &format!(
                "SELECT COALESCE(length(CAST(s.title AS BLOB)),0)
          +COALESCE(length(CAST(s.directory AS BLOB)),0)
          +COALESCE(length(CAST(p.worktree AS BLOB)),0)
          +COALESCE(length(CAST(p.vcs AS BLOB)),0)
        FROM {table} s JOIN project p ON p.id=s.project_id WHERE s.id=?1"
            ),
            [session],
            |row| row.get(0),
        )
        .optional()
        .map_err(AppError::database)?
        .ok_or_else(|| AppError::configuration("OpenCode session disappeared; retry index"))?;
    if metadata_bytes > sqlite_source::MAX_RECORD_BYTES {
        return Err(limits::invalid());
    }
    budget.read(metadata_bytes)?;
    let metadata = connection
        .query_row(
            &format!(
                "SELECT s.title,s.directory,p.worktree,p.vcs,s.time_created,s.time_updated
        FROM {table} s JOIN project p ON p.id=s.project_id WHERE s.id=?1"
            ),
            [session],
            |row| {
                let vcs: Option<String> = row.get("vcs")?;
                Ok(Session {
                    title: row.get("title")?,
                    project: row.get("directory")?,
                    repository: if vcs.as_deref() == Some("git") {
                        Some(row.get("worktree")?)
                    } else {
                        None
                    },
                    created_at: row.get("time_created")?,
                    updated_at: row.get("time_updated")?,
                })
            },
        )
        .optional()
        .map_err(AppError::database)?
        .ok_or_else(|| AppError::configuration("OpenCode session disappeared; retry index"))?;
    if metadata.created_at < 0 || metadata.updated_at < 0 {
        return Err(AppError::configuration(
            "invalid OpenCode session timestamp",
        ));
    }
    let messages = if uses_current(&connection, session)? {
        current(&connection, source, session, &mut budget)?
    } else {
        legacy(&connection, source, session, &mut budget)?
    };
    Ok(ParsedFile {
        chat: Some(Chat {
            id: super::parsers::stable_id(
                "conversation",
                &[Provider::Opencode.as_str(), &source.key()],
            ),
            provider: Provider::Opencode,
            title_origin: None,
            timestamp_origins: None,
            claude_lineage: None,
            subagent: None,
            source: source.clone(),
            title: Some(metadata.title),
            project: (!metadata.project.is_empty()).then_some(metadata.project),
            repository: metadata.repository,
            created_at: Some(metadata.created_at),
            updated_at: Some(metadata.updated_at),
            messages: super::messages::Messages::stage(messages)?,
        }),
        malformed_records: 0,
        first_record_issue: None,
    })
}

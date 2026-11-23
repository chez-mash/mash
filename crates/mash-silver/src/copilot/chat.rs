//! Regular VS Code Chat snapshots and append-only mutation logs, distinct from CLI events.
use super::super::{
    Chat, HistorySource, NormalizedMessage, ParsedFile, messages::Builder, records,
};
use crate::{AppError, message_context::MessageContext, provider::Provider, role::Role};
use serde::de::DeserializeOwned;
use std::{collections::HashSet, fs::File, io::Read, path::Path};

pub mod discovery;
mod models;
mod replay;
pub fn sources(input: Vec<HistorySource>) -> Result<Vec<HistorySource>, AppError> {
    discovery::sources(input)
}

fn invalid(message: &str) -> AppError {
    AppError::configuration(format!("Copilot Chat: {message}"))
}

fn read<T: DeserializeOwned>(path: &Path) -> Result<T, AppError> {
    if !path.metadata().map_err(AppError::io)?.is_file() {
        return Err(invalid("snapshot must be a regular file"));
    }
    let mut bytes = Vec::new();
    let file = File::open(path).map_err(AppError::io)?;
    if !file.metadata().map_err(AppError::io)?.is_file() {
        return Err(invalid("snapshot must be a regular file"));
    }
    file.take((super::super::MAX_RECORD_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(AppError::io)?;
    if bytes.len() > super::super::MAX_RECORD_BYTES {
        return Err(invalid("snapshot exceeds the 8 MiB ingestion limit"));
    }
    records::decode(&bytes)
}

pub fn parse(source: &HistorySource) -> Result<ParsedFile, AppError> {
    super::guarded(|unreadable| snapshot(source, unreadable))
}

fn snapshot(
    source: &HistorySource,
    unreadable: &mut super::Unreadable,
) -> Result<ParsedFile, AppError> {
    let db = &source.identity.path;
    let id = source
        .identity
        .session
        .as_deref()
        .and_then(|s| s.strip_prefix("chat:"))
        .ok_or_else(|| invalid("missing session identity"))?;
    let entry = discovery::catalog(db)?
        .and_then(|mut e| e.remove(id))
        .filter(|e| !e.is_external && !e.is_empty)
        .ok_or_else(AppError::source_changed)?;
    let path = discovery::transcript(db, id)?;
    if !source
        .dependencies
        .iter()
        .any(|d| matches!(d, super::super::source::Dependency::Required(p) if p == &path))
    {
        return Err(AppError::source_changed());
    }
    let session: models::Snapshot = if path.extension().is_some_and(|e| e == "jsonl") {
        replay::read(&path, unreadable)?
    } else {
        read(&path)?
    };
    if session.session_id != id
        || !matches!(session.version, None | Some(2 | 3))
        || session.creation_date < 0
    {
        return Err(invalid(
            "unsupported snapshot version, session identity or timestamp",
        ));
    }
    let project = session
        .working_directory
        .as_deref()
        .or(entry.working_directory.as_deref())
        .and_then(discovery::location)
        .or(discovery::project(db)?);
    let repository = session
        .repo_data
        .and_then(|r| r.remote_url)
        .filter(|s| !s.is_empty());
    let messages = normalize(id, session.requests)?;
    let summary = messages.summary()?;
    Ok(ParsedFile {
        chat: Some(Chat {
            id: super::super::parsers::stable_id("session", &["copilot-chat", id]),
            provider: Provider::Copilot,
            source: source.identity.clone(),
            title_origin: None,
            timestamp_origins: None,
            claude_lineage: None,
            subagent: None,
            title: session
                .custom_title
                .or(session.computed_title)
                .filter(|s| !s.trim().is_empty())
                .or(Some(entry.title).filter(|s| !s.trim().is_empty()))
                .or(summary.title),
            created_at: Some(session.creation_date),
            updated_at: Some(entry.last_message_date.max(summary.updated_at.unwrap_or(0))),
            project,
            repository,
            messages,
        }),
        malformed_records: 0,
        first_record_issue: None,
    })
}

/// One stored message of a request. The prompt and the first passage of the
/// reply keep the identities they have always had; `suffix` names the rest.
struct Row {
    origin: &'static str,
    suffix: Option<String>,
    role: Role,
    content: String,
    projection: String,
    created_at: Option<i64>,
    tools: crate::tools::Activity,
}

/// A response in the order VS Code recorded it: passages of reply text, with
/// thinking, tool calls and uninterpreted parts as messages between them.
struct Reply {
    rows: Vec<Row>,
    visible: bool,
    created_at: Option<i64>,
    passage: Option<Passage>,
    /// Whether a passage has already taken the reply's identity.
    named: bool,
}

/// Reply text being collected: the part it began at, the text, and where the
/// references named in it point.
struct Passage {
    start: usize,
    text: String,
    references: Vec<String>,
}

impl Reply {
    fn context(&mut self, suffix: String, role: Role, content: String) -> &mut Row {
        self.flush();
        self.rows.push(Row {
            origin: "assistant",
            suffix: Some(suffix),
            role,
            content,
            projection: String::new(),
            created_at: self.created_at,
            tools: crate::tools::Activity::default(),
        });
        let last = self.rows.len() - 1;
        &mut self.rows[last]
    }

    fn passage(&mut self, start: usize) -> &mut Passage {
        self.passage.get_or_insert_with(|| Passage {
            start,
            text: String::new(),
            references: Vec::new(),
        })
    }

    fn flush(&mut self) {
        let Some(passage) = self.passage.take() else {
            return;
        };
        if passage.text.trim().is_empty() {
            return;
        }
        let projection = if self.visible {
            crate::dialogue::visible(&Role::Assistant, &passage.text).into_owned()
        } else {
            String::new()
        };
        let mut content = passage.text;
        for line in passage.references {
            content.push('\n');
            content.push_str(&line);
        }
        let suffix =
            std::mem::replace(&mut self.named, true).then(|| format!("text:{}", passage.start));
        self.rows.push(Row {
            origin: "assistant",
            suffix,
            role: Role::Assistant,
            content,
            projection,
            created_at: self.created_at,
            tools: crate::tools::Activity::default(),
        });
    }

    fn part(&mut self, index: usize, part: models::Part) {
        match part.piece() {
            models::Piece::Text(text) => {
                self.passage(index).text.push_str(&text);
            }
            models::Piece::Reference { label, line } => {
                let passage = self.passage(index);
                passage.text.push_str(&label);
                if let Some(line) = line.filter(|line| !passage.references.contains(line)) {
                    passage.references.push(line);
                }
            }
            models::Piece::Thinking(text) => {
                if !text.trim().is_empty() {
                    self.context(
                        format!("thinking:{index}"),
                        Role::Assistant,
                        format!("[Thinking] {text}"),
                    );
                }
            }
            models::Piece::Tool(tool) => {
                let key = tool.call_id.clone().unwrap_or_else(|| index.to_string());
                let name = tool.name.as_deref();
                let mut content = name.map_or_else(|| "Tool".to_owned(), |n| format!("Tool: {n}"));
                if !tool.narration.is_empty() {
                    content.push('\n');
                    content.push_str(&tool.narration);
                }
                self.context(format!("tool:{key}"), Role::Assistant, content)
                    .tools = crate::tools::Activity::call(name, tool.call_id.as_deref(), None);
                if let Some((content, is_error)) = tool.result {
                    self.context(format!("result:{key}"), Role::Tool, content)
                        .tools = crate::tools::Activity::result(tool.call_id.as_deref(), is_error);
                }
            }
            models::Piece::Context(text) => {
                self.context(format!("context:{index}"), Role::Assistant, text);
            }
            models::Piece::Nothing => {}
        }
    }
}

fn normalize(
    id: &str,
    requests: Vec<models::Turn>,
) -> Result<super::super::messages::Messages, AppError> {
    let mut messages = Builder::default();
    let mut identities = HashSet::new();
    let mut ordinal = 0i64;
    for request in requests {
        if request.request_id.trim().is_empty()
            || !identities.insert(request.request_id.clone())
            || request.timestamp.is_some_and(|t| t < 0)
            || request.completed_at().is_some_and(|t| t < 0)
        {
            return Err(invalid("invalid request identity or timestamp"));
        }
        let visible = !request.hidden_from_transcript
            && !request.is_hidden
            && request.agent.as_ref().is_none_or(models::Agent::copilot);
        let completed_at = request.completed_at();
        // A prompt VS Code wrote itself is context, not something the person asked.
        let typed = !request.prompt_hidden && !request.is_system_initiated;
        let user = request.message.text();
        let projection = if visible && typed {
            crate::dialogue::visible(&Role::User, &user).into_owned()
        } else {
            String::new()
        };
        // References stay in the canonical record of the request they were
        // attached to; the projection remains what the person typed.
        let mut content = user;
        if !content.trim().is_empty() {
            for line in request
                .variable_data
                .map_or_else(Vec::new, models::VariableData::references)
            {
                content.push('\n');
                content.push_str(&line);
            }
        }
        let mut reply = Reply {
            rows: vec![Row {
                origin: "user",
                suffix: None,
                role: if typed { Role::User } else { Role::System },
                content,
                projection,
                created_at: request.timestamp,
                tools: crate::tools::Activity::default(),
            }],
            visible,
            created_at: completed_at.or(request.timestamp),
            passage: None,
            named: false,
        };
        for (index, part) in request
            .response
            .map_or_else(Vec::new, models::Response::parts)
            .into_iter()
            .enumerate()
        {
            reply.part(index, part);
        }
        reply.flush();
        for row in reply.rows {
            if row.content.trim().is_empty() {
                continue;
            }
            let mut identity = vec!["copilot-chat", id, &request.request_id, row.origin];
            identity.extend(row.suffix.as_deref());
            messages.push(NormalizedMessage {
                attachments: Vec::new(),
                id: super::super::parsers::stable_id("message", &identity),
                ordinal,
                context: MessageContext::Linear,
                role: row.role,
                content: row.content,
                dialogue_text: Some(row.projection),
                created_at: row.created_at,
                tools: row.tools,
            })?;
            ordinal = ordinal
                .checked_add(1)
                .ok_or_else(|| invalid("too many messages"))?;
        }
    }
    messages.finish()
}

//! VS Code's serializable chat schemas; unrelated editor fields are ignored.
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct Catalog {
    pub version: u32,
    pub entries: BTreeMap<String, Entry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub session_id: String,
    pub title: String,
    pub last_message_date: i64,
    pub working_directory: Option<String>,
    #[serde(default)]
    pub is_external: bool,
    #[serde(default)]
    pub is_empty: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub version: Option<u32>,
    pub session_id: String,
    pub creation_date: i64,
    pub custom_title: Option<String>,
    pub computed_title: Option<String>,
    pub working_directory: Option<String>,
    pub repo_data: Option<Repo>,
    pub requests: Vec<Turn>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
    pub remote_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
// Independent flags mirror the upstream wire schema.
#[allow(clippy::struct_excessive_bools)]
pub struct Turn {
    pub request_id: String,
    pub message: Prompt,
    pub timestamp: Option<i64>,
    pub response_timestamp: Option<i64>,
    pub response: Option<Response>,
    /// Current VS Code records the response's completion here instead of
    /// `responseTimestamp`.
    pub model_state: Option<ModelState>,
    pub agent: Option<Agent>,
    /// What the person attached to the request: files, selections, prompt files.
    pub variable_data: Option<VariableData>,
    #[serde(default)]
    pub is_hidden: bool,
    #[serde(default)]
    pub hidden_from_transcript: bool,
    #[serde(default)]
    #[serde(rename = "requestHiddenFromTranscript")]
    pub prompt_hidden: bool,
    #[serde(default)]
    pub is_system_initiated: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelState {
    pub completed_at: Option<i64>,
}

impl Turn {
    pub fn completed_at(&self) -> Option<i64> {
        self.response_timestamp
            .or_else(|| self.model_state.as_ref().and_then(|s| s.completed_at))
    }
}

#[derive(Default, Deserialize)]
pub struct VariableData {
    #[serde(default)]
    variables: Vec<Variable>,
}

impl VariableData {
    pub fn variables_mut(&mut self) -> &mut Vec<Variable> {
        &mut self.variables
    }

    /// One line per attached file, for the request's canonical content.
    pub fn references(self) -> Vec<String> {
        let mut lines = Vec::new();
        for variable in self.variables {
            let Some(uri) = variable.value.as_ref().and_then(Reference::uri) else {
                continue;
            };
            let line = uri.line(variable.name.as_deref());
            if !lines.contains(&line) {
                lines.push(line);
            }
        }
        lines
    }
}

#[derive(Deserialize)]
pub struct Variable {
    name: Option<String>,
    /// Pasted text, images and other values that name no file are not kept.
    value: Option<Reference>,
}

/// VS Code's serialized `URI`.
#[derive(Deserialize)]
pub struct Uri {
    scheme: String,
    path: String,
    external: Option<String>,
}

impl Uri {
    fn location(&self) -> String {
        self.external
            .clone()
            .unwrap_or_else(|| format!("{}://{}", self.scheme, self.path))
    }

    /// What VS Code shows in place of the reference: a web address in full,
    /// otherwise the file's name.
    fn label(&self) -> String {
        if matches!(self.scheme.as_str(), "http" | "https") {
            return self.location();
        }
        self.path
            .rsplit('/')
            .find(|segment| !segment.is_empty())
            .unwrap_or(&self.path)
            .to_owned()
    }

    fn line(&self, name: Option<&str>) -> String {
        let label = name
            .filter(|name| !name.trim().is_empty())
            .map_or_else(|| self.label(), str::to_owned);
        format!("File {label}: {}", self.location())
    }
}

#[derive(Deserialize)]
pub struct Location {
    uri: Uri,
}

/// A file, a place in one, or a symbol; anything else names nothing Mash keeps.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Reference {
    Symbol { name: String, location: Location },
    Location(Location),
    Uri(Uri),
    Other(serde::de::IgnoredAny),
}

impl Reference {
    fn uri(&self) -> Option<&Uri> {
        match self {
            Self::Symbol { location, .. } | Self::Location(location) => Some(&location.uri),
            Self::Uri(uri) => Some(uri),
            Self::Other(_) => None,
        }
    }

    /// The text VS Code puts in the sentence where the reference stands.
    fn label(&self) -> Option<String> {
        match self {
            Self::Symbol { name, .. } => Some(format!("`{name}`")),
            _ => self.uri().map(Uri::label),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub extension_id: Option<ExtensionId>,
}

#[derive(Deserialize)]
pub struct ExtensionId {
    pub value: String,
}

impl Agent {
    pub fn copilot(&self) -> bool {
        self.extension_id.as_ref().is_none_or(|e| {
            e.value.eq_ignore_ascii_case("github.copilot-chat")
                || e.value.eq_ignore_ascii_case("github.copilot")
        })
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Prompt {
    Text(String),
    Parsed { text: String },
}

impl Prompt {
    pub fn text(self) -> String {
        match self {
            Self::Text(s) | Self::Parsed { text: s } => s,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PartKind {
    MarkdownContent,
    MarkdownVuln,
    Thinking,
    ToolInvocationSerialized,
    InlineReference,
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
pub struct Markdown {
    pub value: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum DisplayText {
    Plain(String),
    Markdown(Markdown),
}

impl DisplayText {
    fn text(self) -> String {
        match self {
            Self::Plain(s) => s,
            Self::Markdown(m) => m.value,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Text {
    String(String),
    Chunks(Vec<String>),
}

impl Text {
    fn text(self) -> String {
        match self {
            Self::String(s) => s,
            Self::Chunks(s) => s.join("\n"),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ResponseInput {
    Text(String),
    Parts(Vec<Part>),
}

#[derive(Default, Deserialize)]
#[serde(from = "ResponseInput")]
pub struct Response {
    parts: Vec<Part>,
}

impl From<ResponseInput> for Response {
    fn from(input: ResponseInput) -> Self {
        Self {
            parts: match input {
                ResponseInput::Text(s) => vec![Part::Text(s)],
                ResponseInput::Parts(p) => p,
            },
        }
    }
}

impl Response {
    pub fn parts(self) -> Vec<Part> {
        self.parts
    }

    pub fn parts_mut(&mut self) -> &mut Vec<Part> {
        &mut self.parts
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Part {
    Text(String),
    Structured(Box<PartData>),
}

/// What one response part contributes to the stored chat.
pub enum Piece {
    /// Reply text. Consecutive pieces are one passage, joined without a
    /// separator as VS Code joins them.
    Text(String),
    /// A file or symbol named inside a sentence: its label continues the
    /// passage, and `line` records where it points.
    Reference {
        label: String,
        line: Option<String>,
    },
    Thinking(String),
    Tool(Tool),
    /// A part kind Mash does not interpret: kept beside the reply, never in it.
    Context(String),
    /// Editor bookkeeping with no text.
    Nothing,
}

pub struct Tool {
    pub name: Option<String>,
    pub call_id: Option<String>,
    /// What VS Code displayed for the call, and its tool-specific details.
    pub narration: String,
    /// The recorded result, with its error flag where the tool reports one.
    pub result: Option<(String, Option<bool>)>,
}

impl Part {
    pub fn piece(self) -> Piece {
        match self {
            Self::Text(s) => Piece::Text(s),
            Self::Structured(part) => part.piece(),
        }
    }
}

#[derive(Deserialize)]
pub struct PartData {
    pub kind: Option<PartKind>,
    pub value: Option<Text>,
    pub content: Option<Markdown>,
    pub text: Option<String>,
    #[serde(rename = "inlineReference")]
    pub inline_reference: Option<Reference>,
    #[serde(rename = "toolId")]
    pub tool_id: Option<String>,
    #[serde(rename = "toolCallId")]
    pub tool_call_id: Option<String>,
    #[serde(rename = "invocationMessage")]
    pub invocation_message: Option<DisplayText>,
    #[serde(rename = "pastTenseMessage")]
    pub past_tense_message: Option<DisplayText>,
    // Tool results and details have an extensible provider-owned schema,
    // never searchable.
    #[serde(rename = "resultDetails")]
    pub result_details: Option<serde_json::Value>,
    #[serde(rename = "toolSpecificData")]
    pub tool_specific_data: Option<serde_json::Value>,
}

impl PartData {
    pub fn piece(self) -> Piece {
        let text = self
            .value
            .map(Text::text)
            .or_else(|| self.content.map(|m| m.value))
            .or(self.text)
            .unwrap_or_default();
        match self.kind {
            None | Some(PartKind::MarkdownContent | PartKind::MarkdownVuln) => Piece::Text(text),
            Some(PartKind::Thinking) => Piece::Thinking(text),
            Some(PartKind::InlineReference) => {
                let reference = self.inline_reference;
                match reference.as_ref().and_then(Reference::label) {
                    Some(label) => Piece::Reference {
                        label,
                        // A web address already stands in the sentence in full.
                        line: reference
                            .as_ref()
                            .and_then(Reference::uri)
                            .filter(|uri| uri.label() != uri.location())
                            .map(|uri| uri.line(None)),
                    },
                    None => Piece::Nothing,
                }
            }
            Some(PartKind::ToolInvocationSerialized) => {
                let narration = [
                    Some(text),
                    self.invocation_message.map(DisplayText::text),
                    self.past_tense_message.map(DisplayText::text),
                    self.tool_specific_data.map(|v| v.to_string()),
                ]
                .into_iter()
                .flatten()
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
                Piece::Tool(Tool {
                    name: self.tool_id,
                    call_id: self.tool_call_id,
                    narration,
                    result: self.result_details.map(|details| {
                        let is_error = details.get("isError").and_then(serde_json::Value::as_bool);
                        (details.to_string(), is_error)
                    }),
                })
            }
            Some(PartKind::Other) if text.is_empty() => Piece::Nothing,
            Some(PartKind::Other) => Piece::Context(text),
        }
    }
}

#[derive(Deserialize)]
pub struct Workspace {
    pub folder: Option<String>,
}

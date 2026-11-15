//! Selected dialogue and identity fields from the pinned IDE 2.5.5 descriptors,
//! unchanged in the Antigravity 2.31.2 app's embedded descriptors.
pub use super::super::native::{Timestamp, Workspace};
use prost::Message;

#[derive(Clone, Message)]
pub struct Step {
    #[prost(enumeration = "Kind", tag = "1")]
    pub kind: i32,
    #[prost(enumeration = "Status", tag = "4")]
    pub status: i32,
    #[prost(message, optional, tag = "5")]
    pub metadata: Option<StepMetadata>,
    #[prost(message, optional, tag = "19")]
    pub user: Option<User>,
    #[prost(message, optional, tag = "20")]
    pub assistant: Option<Assistant>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
#[repr(i32)]
pub enum Kind {
    Unspecified = 0,
    User = 14,
    Assistant = 15,
    // Tool steps: each holds one tool's outcome, in the body `outcome.rs`
    // names, and answers the call in the step metadata.
    GrepSearch = 7,
    ViewFile = 8,
    ListDirectory = 9,
    RunCommand = 21,
    Find = 25,
    CommandStatus = 28,
    ReadUrlContent = 31,
    ViewContentChunk = 32,
    SearchWeb = 33,
    McpTool = 38,
    ViewFileOutline = 47,
    ListResources = 51,
    ReadResource = 52,
    OpenBrowserUrl = 56,
    RunExtensionCode = 57,
    ExecuteBrowserJavascript = 61,
    ClickBrowserPixel = 64,
    ReadTerminal = 65,
    CaptureBrowserConsoleLogs = 66,
    ReadBrowserPage = 67,
    BrowserGetDom = 68,
    BrowserInput = 74,
    BrowserMoveMouse = 75,
    BrowserSelectOption = 76,
    BrowserScrollUp = 77,
    BrowserScrollDown = 78,
    BrowserClickElement = 79,
    BrowserPressKey = 80,
    BrowserSubagent = 85,
    BrowserScroll = 88,
    CompileApplet = 93,
    InstallAppletDependencies = 94,
    InstallAppletPackage = 95,
    BrowserResizeWindow = 96,
    SendCommandInput = 100,
    ShellExec = 112,
    BrowserMouseWheel = 113,
    LintApplet = 114,
    BrowserMouseUp = 120,
    BrowserMouseDown = 121,
    WorkspaceApi = 122,
    BrowserListNetworkRequests = 123,
    BrowserGetNetworkRequest = 124,
    BrowserRefreshPage = 125,
    ReadNotebook = 129,
    Generic = 132,
    ExecuteNotebook = 134,
    CloudSqlUpdateSchema = 135,
    CloudSqlExecuteSql = 137,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
#[repr(i32)]
pub enum Status {
    Unspecified = 0,
    Done = 3,
    Error = 7,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
#[repr(i32)]
pub enum TrajectoryKind {
    Unspecified = 0,
    Cascade = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
#[repr(i32)]
pub enum TrajectorySource {
    Unspecified = 0,
    CascadeClient = 1,
    /// Conversations started in the Antigravity 2.31 app's agent manager.
    GeminiApp = 21,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
#[repr(i32)]
pub enum Source {
    Unspecified = 0,
    Manual = 1,
    Model = 2,
    UserImplicit = 3,
    UserExplicit = 4,
    System = 5,
    SystemSdk = 6,
}
#[derive(Clone, Message)]
pub struct StepMetadata {
    #[prost(message, optional, tag = "1")]
    pub created_at: Option<Timestamp>,
    #[prost(enumeration = "Source", tag = "3")]
    pub source: i32,
    /// The call a tool step answers.
    #[prost(message, optional, tag = "4")]
    pub tool_call: Option<ToolCall>,
    #[prost(message, optional, tag = "20")]
    pub origin: Option<Origin>,
}
#[derive(Clone, Message)]
pub struct Origin {
    #[prost(string, tag = "1")]
    pub trajectory: String,
    #[prost(uint32, tag = "2")]
    pub step: u32,
    #[prost(string, tag = "4")]
    pub cascade: String,
}
#[derive(Clone, Message)]
pub struct User {
    #[prost(string, tag = "1")]
    pub legacy_query: String,
    #[prost(string, tag = "2")]
    pub response: String,
    #[prost(message, repeated, tag = "3")]
    pub items: Vec<Item>,
}
#[derive(Clone, Message)]
pub struct Item {
    #[prost(string, tag = "1")]
    pub text: String,
    #[prost(message, optional, tag = "2")]
    pub scope: Option<Scope>,
}
#[derive(Clone, Message)]
pub struct Scope {
    #[prost(message, optional, tag = "11")]
    pub text_block: Option<TextBlock>,
}
#[derive(Clone, Message)]
pub struct TextBlock {
    #[prost(string, tag = "1")]
    pub content: String,
    #[prost(string, tag = "3")]
    pub label: String,
}
#[derive(Clone, Message)]
pub struct Assistant {
    #[prost(string, tag = "1")]
    pub response: String,
    #[prost(string, tag = "3")]
    pub thinking: String,
    #[prost(message, repeated, tag = "7")]
    pub tool_calls: Vec<ToolCall>,
    #[prost(string, tag = "8")]
    pub modified_response: String,
    #[prost(string, tag = "16")]
    pub raw_thinking: String,
}
#[derive(Clone, Message)]
pub struct ToolCall {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, tag = "3")]
    pub arguments_json: String,
}
#[derive(Clone, Message)]
pub struct Metadata {
    #[prost(message, repeated, tag = "1")]
    pub workspaces: Vec<Workspace>,
    #[prost(message, optional, tag = "2")]
    pub created_at: Option<Timestamp>,
    #[prost(string, tag = "5")]
    pub parent: String,
    #[prost(string, tag = "6")]
    pub root: String,
    #[prost(string, repeated, tag = "7")]
    pub workspace_uris: Vec<String>,
    #[prost(message, optional, tag = "8")]
    pub subagent: Option<Subagent>,
}
#[derive(Clone, Message)]
pub struct Subagent {}

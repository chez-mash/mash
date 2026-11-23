//! Recording each MCP tool call in the usage log, once, around the SDK's own
//! dispatch: the tool's name and arguments as the client sent them, the
//! client's name and version from its `initialize` handshake, how the call
//! ended, and references to what it returned. One stdio session is one process, so the process start names the
//! connection.
use std::future::Future;
use std::sync::OnceLock;

use mash_app::core::{self, Paths};
use mash_app::usage::{self, Outcome, Started, Surface};
use rmcp::model::{CallToolRequestParams, CallToolResponse, ContentBlock};

/// This connection's ID, the same for every call it serves.
fn session() -> &'static str {
    static SESSION: OnceLock<String> = OnceLock::new();
    SESSION.get_or_init(|| {
        let started = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_millis());
        format!("mcp-{}-{started}", std::process::id())
    })
}

pub(crate) async fn recorded<F, Fut>(
    paths: &Paths,
    request: CallToolRequestParams,
    context: rmcp::service::RequestContext<rmcp::RoleServer>,
    call: F,
) -> Result<CallToolResponse, rmcp::ErrorData>
where
    F: FnOnce(CallToolRequestParams, rmcp::service::RequestContext<rmcp::RoleServer>) -> Fut,
    Fut: Future<Output = Result<CallToolResponse, rmcp::ErrorData>>,
{
    let started = Started::now();
    let tool = request.name.to_string();
    let arguments = request.arguments.clone().map_or_else(
        || serde_json::Value::Object(serde_json::Map::new()),
        serde_json::Value::Object,
    );
    let client = context.peer.peer_info().map(|info| {
        (
            info.client_info.name.clone(),
            info.client_info.version.clone(),
        )
    });
    let response = call(request, context).await;
    let mut row = started.call(Surface::Mcp, &tool, arguments);
    if let Some((name, version)) = client {
        row.client = Some(name);
        row.client_version = Some(version);
    }
    row.session = Some(session().to_owned());
    match &response {
        Ok(CallToolResponse::Complete(result)) => {
            let texts = result.content.iter().filter_map(ContentBlock::as_text);
            row.response_bytes = Some(texts.map(|text| text.text.len() as u64).sum());
            let first = result
                .content
                .iter()
                .find_map(ContentBlock::as_text)
                .map(|text| text.text.as_str());
            if result.is_error == Some(true) {
                row.outcome = Outcome::Error;
                failed(&mut row, first);
            } else {
                // The writer reads what the call returned from this, off the
                // client's path; the text itself is never stored.
                row.response = result.structured_content.clone().map(usage::Returned::Json);
            }
        }
        Ok(_) => {}
        Err(error) => {
            row.outcome = Outcome::Error;
            row.error_message = Some(error.message.to_string());
        }
    }
    let database = paths.db.clone().unwrap_or_else(core::default_database_path);
    usage::record(&database, row);
    response
}

/// A failed tool result is Mash's JSON error; take its kind and message, or
/// the text as the message when it is not JSON.
fn failed(row: &mut usage::Call, text: Option<&str>) {
    let Some(text) = text else {
        return;
    };
    let parsed: Option<serde_json::Value> = serde_json::from_str(text).ok();
    let error = parsed.as_ref().and_then(|value| value.get("error"));
    row.error_kind = error
        .and_then(|error| error.get("kind"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    row.error_message = Some(
        error
            .and_then(|error| error.get("message"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or(text)
            .to_owned(),
    );
}

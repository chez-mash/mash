//! Where each tool step type records its outcome, by field number in the
//! pinned descriptors. A step holds its type's body in one field of its own;
//! only the fields named here are read, and everything else in a body is
//! skipped.
use super::{invalid, proto::Kind};
use crate::AppError;
use prost::encoding::{DecodeContext, WireType, decode_key, decode_varint, skip_field};

/// Field numbers from a message down to one of its values.
type Path = &'static [u32];

/// The step-level error details (step field 31): the message given to the
/// model, then the one shown to the user, then the short form.
const STEP_ERROR: u32 = 31;
const STEP_ERROR_TEXT: &[Path] = &[&[9], &[1], &[2]];

/// One step type's outcome fields, as paths within its body.
struct Layout {
    /// The step field holding this type's body.
    body: u32,
    /// Result text: the first candidate with any text. A repeated value
    /// contributes one line per occurrence.
    text: &'static [Path],
    /// Error text. Any marks the result failed, and stands in for result
    /// text when there is none.
    error: &'static [Path],
    /// Flags and exit codes; a non-zero value marks the result failed.
    failed: &'static [Path],
}
impl Layout {
    const fn error(mut self, error: &'static [Path]) -> Self {
        self.error = error;
        self
    }
    const fn failed(mut self, failed: &'static [Path]) -> Self {
        self.failed = failed;
        self
    }
}
const fn text(body: u32, text: &'static [Path]) -> Layout {
    Layout {
        body,
        text,
        error: &[],
        failed: &[],
    }
}

fn layout(kind: Kind) -> Option<Layout> {
    Some(match kind {
        Kind::Unspecified | Kind::User | Kind::Assistant => return None,
        // result.result
        Kind::Generic => text(140, &[&[2, 1]]),
        // Files and searches.
        // content, raw_content
        Kind::ViewFile => text(14, &[&[4], &[9]]),
        // children, results.name; dir_not_found
        Kind::ListDirectory => text(15, &[&[2], &[3, 1]]).failed(&[&[4]]),
        // raw_output; grep_error
        Kind::GrepSearch => text(13, &[&[3]]).error(&[&[5]]),
        // raw_output, truncated_output; find_error
        Kind::Find => text(34, &[&[11], &[14]]).error(&[&[8]]),
        // contents, outline_items
        Kind::ViewFileOutline => text(58, &[&[7], &[9]]),
        // Commands.
        // combined_output.full, combined_output.truncated, stdout, stderr;
        // exit_code, user_rejected
        Kind::RunCommand => text(28, &[&[21, 1], &[21, 2], &[4], &[5]]).failed(&[&[6], &[14]]),
        // combined, delta, stdout, stderr; exit_code
        Kind::CommandStatus => text(37, &[&[9], &[12], &[3], &[4]]).failed(&[&[5]]),
        // output.full, output.truncated; exit_code, user_rejected
        Kind::SendCommandInput => text(113, &[&[8, 1], &[8, 2]]).failed(&[&[10], &[4]]),
        // contents
        Kind::ReadTerminal => text(77, &[&[3]]),
        // output; error_message; exit_code
        Kind::ShellExec => text(127, &[&[3]]).error(&[&[4]]).failed(&[&[2]]),
        // output; user_rejected
        Kind::RunExtensionCode => text(68, &[&[3]]).failed(&[&[4]]),
        // MCP, the web and workspace services.
        // result_string; user_rejected
        Kind::McpTool => text(47, &[&[3]]).failed(&[&[7]]),
        // resources.uri
        Kind::ListResources => text(62, &[&[3, 1]]),
        // contents.text.text
        Kind::ReadResource => text(63, &[&[3, 2, 1]]),
        // web_document.text, web_document.summary; user_rejected
        Kind::ReadUrlContent => text(40, &[&[2, 2], &[2, 7]]).failed(&[&[5]]),
        // cropped_item.text
        Kind::ViewContentChunk => text(41, &[&[4, 2]]),
        // summary
        Kind::SearchWeb => text(42, &[&[5]]),
        // response
        Kind::WorkspaceApi => text(136, &[&[6]]),
        // The browser: what a page returned, else how the action changed it
        // (browser_state_diff).
        // result
        Kind::BrowserSubagent => text(97, &[&[2]]),
        // javascript_result, browser_state_diff; user_rejected
        Kind::ExecuteBrowserJavascript => text(73, &[&[8], &[11]]).failed(&[&[4]]),
        // web_document.text
        Kind::ReadBrowserPage => text(79, &[&[2, 2]]),
        // serialized_dom_tree
        Kind::BrowserGetDom => text(80, &[&[3]]),
        // console_logs.lines.output
        Kind::CaptureBrowserConsoleLogs => text(78, &[&[3, 1, 3]]),
        // network_requests
        Kind::BrowserListNetworkRequests => text(137, &[&[5]]),
        // network_request_details
        Kind::BrowserGetNetworkRequest => text(138, &[&[4]]),
        Kind::OpenBrowserUrl => text(67, &[&[9]]).failed(&[&[3]]),
        Kind::ClickBrowserPixel => text(76, &[&[8]]).failed(&[&[4]]),
        Kind::BrowserClickElement => text(91, &[&[7]]).failed(&[&[4]]),
        Kind::BrowserResizeWindow => text(109, &[&[7]]).failed(&[&[4]]),
        Kind::BrowserInput => text(86, &[&[7]]),
        Kind::BrowserMoveMouse => text(87, &[&[7]]),
        Kind::BrowserSelectOption => text(88, &[&[7]]),
        Kind::BrowserScrollUp => text(89, &[&[5]]),
        Kind::BrowserScrollDown => text(90, &[&[5]]),
        Kind::BrowserPressKey => text(92, &[&[4]]),
        Kind::BrowserScroll => text(101, &[&[8]]),
        Kind::BrowserMouseWheel => text(125, &[&[7]]),
        Kind::BrowserMouseUp => text(134, &[&[4]]),
        Kind::BrowserMouseDown => text(135, &[&[4]]),
        Kind::BrowserRefreshPage => text(139, &[&[3]]),
        // Notebooks.
        // reply.flattened_content
        Kind::ReadNotebook => text(144, &[&[3, 1]]),
        // reply.cell_outputs.output_text; reply.error_trace,
        // reply.cell_outputs.error_trace
        Kind::ExecuteNotebook => text(150, &[&[3, 3, 4]]).error(&[&[3, 4], &[3, 3, 7]]),
        // Hosted applets and their databases.
        // logs; error_message
        Kind::CompileApplet => text(106, &[&[3]]).error(&[&[1]]),
        Kind::InstallAppletDependencies => text(107, &[&[2]]).error(&[&[1]]),
        Kind::InstallAppletPackage => text(108, &[&[3]]).error(&[&[2]]),
        // output; error_message; exit_code
        Kind::LintApplet => text(126, &[&[2]]).error(&[&[3]]).failed(&[&[1]]),
        // output; error_message
        Kind::CloudSqlUpdateSchema => text(151, &[&[4]]).error(&[&[1]]),
        Kind::CloudSqlExecuteSql => text(153, &[&[5]]).error(&[&[4]]),
    })
}

/// One tool's outcome as its step records it.
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    pub text: String,
    /// The body itself reports a failure; the step status may report one too.
    pub failed: bool,
}

enum Value<'a> {
    Number(u64),
    Bytes(&'a [u8]),
}

fn delimited<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8], AppError> {
    let size =
        usize::try_from(decode_varint(bytes).map_err(|_| invalid("invalid tool result length"))?)
            .map_err(|_| invalid("oversized tool result length"))?;
    let (value, rest) = bytes
        .split_at_checked(size)
        .ok_or_else(|| invalid("truncated tool result"))?;
    *bytes = rest;
    Ok(value)
}

/// Visits every value at `path` under `message`, in stored order.
fn walk<'a>(
    mut message: &'a [u8],
    path: &[u32],
    visit: &mut dyn FnMut(Value<'a>) -> Result<(), AppError>,
) -> Result<(), AppError> {
    let Some((&field, rest)) = path.split_first() else {
        return Ok(());
    };
    while !message.is_empty() {
        let (tag, wire) =
            decode_key(&mut message).map_err(|_| invalid("invalid tool result key"))?;
        match wire {
            WireType::LengthDelimited if tag == field => {
                let value = delimited(&mut message)?;
                if rest.is_empty() {
                    visit(Value::Bytes(value))?;
                } else {
                    walk(value, rest, visit)?;
                }
            }
            WireType::Varint if tag == field && rest.is_empty() => visit(Value::Number(
                decode_varint(&mut message).map_err(|_| invalid("invalid tool result number"))?,
            ))?,
            _ => skip_field(wire, tag, &mut message, DecodeContext::default())
                .map_err(|_| invalid("invalid tool result field"))?,
        }
    }
    Ok(())
}

/// A step's one message at `field`. Bodies are alternatives of one step, so
/// a second occurrence is malformed.
fn single(step: &[u8], field: u32) -> Result<Option<&[u8]>, AppError> {
    let mut found = None;
    walk(step, &[field], &mut |value| {
        if let Value::Bytes(bytes) = value
            && found.replace(bytes).is_some()
        {
            return Err(invalid("duplicate native step field"));
        }
        Ok(())
    })?;
    Ok(found)
}

/// The first candidate holding any text, its occurrences joined by lines.
/// Each occurrence counts against the structure limit.
fn first(
    message: &[u8],
    candidates: &[Path],
    remaining: &mut usize,
) -> Result<Option<String>, AppError> {
    for path in candidates {
        let mut lines = Vec::new();
        walk(message, path, &mut |value| {
            if let Value::Bytes(bytes) = value {
                *remaining = remaining
                    .checked_sub(1)
                    .ok_or_else(|| invalid("protobuf field limit exceeded"))?;
                lines.push(
                    std::str::from_utf8(bytes).map_err(|_| invalid("invalid tool result text"))?,
                );
            }
            Ok(())
        })?;
        let text = lines.join("\n");
        if !text.trim().is_empty() {
            return Ok(Some(text));
        }
    }
    Ok(None)
}

fn flagged(message: &[u8], flags: &[Path]) -> Result<bool, AppError> {
    let mut set = false;
    for path in flags {
        walk(message, path, &mut |value| {
            set |= matches!(value, Value::Number(number) if number != 0);
            Ok(())
        })?;
    }
    Ok(set)
}

/// The outcome a stored step of this type records, if it is a tool step.
pub fn read(kind: i32, step: &[u8], remaining: &mut usize) -> Result<Option<Outcome>, AppError> {
    let Some(layout) = Kind::try_from(kind).ok().and_then(layout) else {
        return Ok(None);
    };
    let body = single(step, layout.body)?.unwrap_or_default();
    let error = first(body, layout.error, remaining)?;
    let failed = error.is_some() || flagged(body, layout.failed)?;
    let text = match first(body, layout.text, remaining)?.or(error) {
        Some(text) => text,
        None => match single(step, STEP_ERROR)? {
            Some(details) => first(details, STEP_ERROR_TEXT, remaining)?.unwrap_or_default(),
            None => String::new(),
        },
    };
    Ok(Some(Outcome { text, failed }))
}

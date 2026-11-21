//! The `mash` and `mash-server` executables: argument parsing and output over
//! the typed application library, `mash-app`.
// The search futures this crate drives nest deeper than the default limit
// when the compiler proves them `Send`, as they do in `mash-app`.
#![recursion_limit = "256"]
mod cli;
mod server_cli;

use std::ffi::OsString;
use std::io::{self, IsTerminal, Write};

use mash_app::{
    BROKEN_PIPE, blocking, chat_catalog, config, core, observability, remote, runtime,
    search_filter, search_results, semantic, shared, storage, views, watch, write_json,
};
use mash_dash as dash;
use mash_mcp as mcp;
use mash_model::{AppError, platform, provider, redaction, role};
use mash_server as server;
use serde::Serialize;

/// Registers the silver text codec on a raw connection, as tests need to read silver directly.
#[doc(hidden)]
pub use mash_app::register_silver_codec;
/// Build-time generation of the bundled history skill from the live MCP contract.
pub use mash_mcp::guidance::{generate_history_skill, validate_history_release};

fn mash_description() -> &'static str {
    if dash::has_web_dashboard() {
        "Search saved coding-agent chats. Open the browser dashboard with mash dash, \
         or connect an agent with mash init."
    } else {
        "Search saved coding-agent chats from the terminal with mash search, \
         or connect an agent with mash init."
    }
}

/// Runs mash with the process arguments, writing JSON or serving stdio MCP.
///
/// # Examples
///
/// ```no_run
/// let exit_code = mash::main_entry();
/// std::process::exit(exit_code);
/// ```
#[must_use]
pub fn main_entry() -> i32 {
    observability::initialize();
    if let Some(code) = semantic::remote_reranker::managed::entry(std::env::args_os()) {
        return code;
    }
    let mut stdout = io::stdout();
    let terminal = stdout.is_terminal();
    execute(
        std::env::args_os(),
        &mut stdout,
        &mut io::stderr(),
        terminal,
    )
}

/// Run an entry point on a thread with an 8 MiB stack, the Linux default.
/// Windows gives the main thread 1 MiB, which `mash index` with shared
/// publication overflowed in debug builds (`STATUS_STACK_OVERFLOW`,
/// 0xc00000fd): their async state machines are large. The thread is named
/// `main` so messages read as before, and a panic exits with 101, as Rust's
/// own main does.
pub fn with_main_stack(entry: fn() -> i32) -> i32 {
    const STACK_BYTES: usize = 8 * 1024 * 1024;
    match std::thread::Builder::new()
        .name("main".to_owned())
        .stack_size(STACK_BYTES)
        .spawn(entry)
    {
        Ok(thread) => thread.join().unwrap_or(101),
        // Without a thread, run where we are rather than not at all.
        Err(_) => entry(),
    }
}

/// Runs the standalone shared-index HTTP server with process arguments.
#[must_use]
pub fn server_main_entry() -> i32 {
    observability::initialize();
    if let Some(code) = semantic::remote_reranker::managed::entry(std::env::args_os()) {
        return code;
    }
    match server_cli::run(std::env::args_os()) {
        Ok(Some(server_cli::Information::Help { help })) => {
            write_text(&mut io::stdout(), &mut io::stderr(), &help)
        }
        Ok(Some(response)) => write_json(&mut io::stdout(), &response).map_or_else(
            |error| {
                if error.code == BROKEN_PIPE {
                    0
                } else {
                    error.code
                }
            },
            |()| 0,
        ),
        Ok(None) => 0,
        Err(error) => {
            let _ = write_json(&mut io::stderr(), &error);
            error.code
        }
    }
}

fn execute<I, T>(args: I, stdout: &mut dyn Write, stderr: &mut dyn Write, terminal: bool) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let arguments: Vec<OsString> = args.into_iter().map(Into::into).collect();
    match cli::run(arguments.clone()) {
        // MCP owns stdout for its entire session, including shutdown.
        Ok(cli::Response::ServerStopped) => 0,
        // The async runtime and its lease are gone; start the replacement.
        Ok(cli::Response::WatchReplaced { reason, executable }) => {
            let _ = write_json(stderr, &reason);
            if cfg!(unix) {
                let _ = write_json(stderr, &WatchRestart::event());
                let error = runtime::restart(&executable, &arguments);
                let _ = write_json(stderr, &error);
                error.code
            } else {
                reason.code
            }
        }
        // Help is for a person to read: plain text, as clap renders it.
        Ok(cli::Response::Help { help }) => write_text(stdout, stderr, &help),
        Ok(cli::Response::Setup(value)) if terminal => {
            let core::Response::Init(report) = value.as_ref() else {
                let error = AppError::internal("wizard did not return a setup report");
                let _ = write_json(stderr, &error);
                return error.code;
            };
            let summary = report.terminal_summary(dash::has_web_dashboard());
            write_summary(stdout, stderr, &value, &summary)
        }
        Ok(cli::Response::InitCancelled { .. }) if terminal => 0,
        Ok(response) => {
            if terminal
                && let cli::Response::Operation(value) = &response
                && let core::Response::Uninstall(report) = value.as_ref()
            {
                return write_summary(stdout, stderr, value, &report.terminal_summary());
            }
            match write_json(stdout, &response) {
                // A reader that stopped early (`mash ... | head`) is not a failure.
                Err(error) if error.code == BROKEN_PIPE => 0,
                Err(error) => {
                    let _ = write_json(stderr, &error);
                    error.code
                }
                Ok(()) => match response {
                    cli::Response::Operation(value) | cli::Response::Setup(value) => {
                        operation_exit(&value, Some(stderr))
                    }
                    _ => 0,
                },
            }
        }
        Err(error) if error.code == BROKEN_PIPE => 0,
        Err(error) => {
            let _ = write_json(stderr, &error);
            error.code
        }
    }
}

fn write_summary(
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    value: &core::Response,
    summary: &str,
) -> i32 {
    let written = write_text(stdout, stderr, summary);
    if written == 0 {
        operation_exit(value, None)
    } else {
        written
    }
}

fn operation_exit(value: &core::Response, error_output: Option<&mut dyn Write>) -> i32 {
    if let Some(error) = value.failure() {
        if let Some(output) = error_output {
            let _ = write_json(output, error);
        }
        error.code
    } else if value.needs_retry() {
        9
    } else {
        i32::from(value.reports_problem())
    }
}

/// Logged before a replaced watcher starts its replacement in place.
#[derive(Serialize)]
struct WatchRestart {
    event: &'static str,
    reason: &'static str,
}

impl WatchRestart {
    const fn event() -> Self {
        Self {
            event: "watch-restart",
            reason: "executable-replaced",
        }
    }
}

/// Prints terminal text and returns the exit code. A reader that stopped early
/// (`mash --help | head`) is not a failure.
fn write_text(stdout: &mut dyn Write, stderr: &mut dyn Write, text: &str) -> i32 {
    let text = text.trim_end();
    match writeln!(stdout, "{text}").and_then(|()| stdout.flush()) {
        Ok(()) => 0,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => 0,
        Err(error) => {
            let error = AppError::io(error);
            let _ = write_json(stderr, &error);
            error.code
        }
    }
}

#[cfg(test)]
mod output_tests {
    use std::io::{self, Write};

    /// A reader that has gone away, as with `mash ... | head`.
    struct Closed;

    impl Write for Closed {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_closed_output_pipe_exits_quietly() {
        let mut stderr = Vec::new();
        let code = super::execute(["mash", "--version"], &mut Closed, &mut stderr, false);
        assert_eq!(code, 0);
        assert!(stderr.is_empty(), "{}", String::from_utf8_lossy(&stderr));
    }
}

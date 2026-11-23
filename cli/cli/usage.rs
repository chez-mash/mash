//! `mash usage on|off|clear`, and recording each command this program runs in
//! the usage log. Mash only keeps the log; it is a plain SQLite file to query
//! with any SQLite tool. A command is recorded once, here at its boundary, with the arguments
//! Clap parsed. `mash mcp` and `mash dash` are not recorded as commands: they
//! are servers, and record each tool call or request they serve instead.
use std::ffi::OsString;
use std::io::IsTerminal;
use std::path::PathBuf;

use clap::{ArgMatches, CommandFactory, Subcommand};
use serde::Serialize;

use super::{Cli, Response};
use crate::AppError;
use crate::usage::{self, Started, Surface};

#[derive(Debug, Clone, Subcommand)]
pub(super) enum UsageCommand {
    /// Delete every recorded call. Asks first in a terminal.
    Clear {
        /// Delete without asking.
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Record calls again (the default).
    On,
    /// Stop recording calls; what is recorded stays until cleared.
    Off,
}

/// What a `mash usage` command answers.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum Answer {
    Cleared { deleted: u64 },
    Recording { recording: bool, path: String },
}

fn database(cli: &Cli) -> PathBuf {
    cli.db
        .clone()
        .unwrap_or_else(crate::core::default_database_path)
}

/// Run a `mash usage` command.
pub(super) fn run(cli: &Cli, command: &UsageCommand) -> Result<Response, AppError> {
    let database = database(cli);
    let answer = match command {
        UsageCommand::Clear { yes } => {
            if !yes && !confirm_clear()? {
                return Ok(Response::Usage(Answer::Cleared { deleted: 0 }));
            }
            Answer::Cleared {
                deleted: usage::clear(&database)?,
            }
        }
        UsageCommand::On | UsageCommand::Off => {
            let on = matches!(command, UsageCommand::On);
            usage::set_enabled(&database, on)?;
            Answer::Recording {
                recording: usage::enabled(&database)?,
                path: usage::path_for(&database).display().to_string(),
            }
        }
    };
    Ok(Response::Usage(answer))
}

fn confirm_clear() -> Result<bool, AppError> {
    if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal() {
        return Err(AppError::usage(
            "mash usage clear deletes every recorded call; pass --yes to confirm",
        ));
    }
    eprint!("Delete every recorded call from the usage log? This cannot be undone. [y/N] ");
    let mut answer = String::new();
    std::io::stdin()
        .read_line(&mut answer)
        .map_err(AppError::io)?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes"))
}

/// A command being recorded: when it started, what it was, and where its log
/// lives.
pub(crate) struct Pending {
    started: Started,
    database: PathBuf,
    tool: String,
    arguments: serde_json::Value,
}

/// Start recording the command `arguments` names, unless it is a server
/// (`mcp`, `dash`) or `usage` itself, or does not parse.
pub(crate) fn start(arguments: &[OsString]) -> Option<Pending> {
    let started = Started::now();
    let matches = Cli::command().try_get_matches_from(arguments).ok()?;
    let mut names = Vec::new();
    let mut fields = serde_json::Map::new();
    let mut level = &matches;
    loop {
        collect(level, &mut fields);
        match level.subcommand() {
            Some((name, next)) => {
                names.push(name.to_owned());
                level = next;
            }
            None => break,
        }
    }
    if matches!(
        names.first().map(String::as_str),
        Some("mcp" | "dash" | "usage")
    ) {
        return None;
    }
    let database = matches
        .get_one::<PathBuf>("db")
        .cloned()
        .unwrap_or_else(crate::core::default_database_path);
    Some(Pending {
        started,
        database,
        tool: names.join(" "),
        arguments: serde_json::Value::Object(fields),
    })
}

/// Every argument given at one level, by its name, as text.
fn collect(matches: &ArgMatches, fields: &mut serde_json::Map<String, serde_json::Value>) {
    for id in matches.ids() {
        let name = id.as_str();
        if matches.value_source(name) != Some(clap::parser::ValueSource::CommandLine) {
            continue;
        }
        let Ok(Some(raw)) = matches.try_get_raw(name) else {
            continue;
        };
        let values: Vec<serde_json::Value> = raw
            .map(|value| serde_json::Value::String(value.to_string_lossy().into_owned()))
            .collect();
        let value = match values.len() {
            0 => serde_json::Value::Bool(true),
            1 => values.into_iter().next().unwrap_or(serde_json::Value::Null),
            _ => serde_json::Value::Array(values),
        };
        fields.insert(name.to_owned(), value);
    }
}

impl Pending {
    /// Record the command's outcome, how many bytes it printed, and the
    /// printed JSON, from which the writer notes what the command returned.
    pub(crate) fn finish(
        self,
        result: &Result<Response, AppError>,
        bytes: Option<u64>,
        printed: Option<String>,
    ) {
        let mut call = self.started.call(Surface::Cli, &self.tool, self.arguments);
        call.client = usage::client_from_environment();
        call.response_bytes = bytes;
        call.response = printed.map(usage::Returned::Text);
        match result {
            Ok(Response::Operation(response) | Response::Setup(response)) => {
                call.answered(response);
            }
            Ok(_) => {}
            Err(error) => call.fail(error),
        }
        usage::record(&self.database, call);
    }
}

//! The global paths `--help` reports, read without the normal parser.
use super::{Cli, Paths, Response, VERSION_INFO};
use clap::CommandFactory;
use std::ffi::OsString;
use std::path::PathBuf;

// Parse global paths with Clap even when help short-circuits the normal parser.
// Values containing `--help` remain values; no token stripping or hand parsing.
pub(super) fn help_paths(arguments: &[OsString]) -> Paths {
    fn passive(command: clap::Command) -> clap::Command {
        command
            .disable_help_flag(true)
            .disable_help_subcommand(true)
            .arg_required_else_help(false)
            .subcommand_required(false)
            .mut_subcommands(passive)
    }
    let command = passive(Cli::command())
        .arg(
            clap::Arg::new("diagnostic-help")
                .long("help")
                .short('h')
                .global(true)
                .action(clap::ArgAction::SetTrue),
        )
        .subcommand(clap::Command::new("help").arg(clap::Arg::new("topic").num_args(0..)))
        .ignore_errors(true);
    command.try_get_matches_from(arguments).map_or_else(
        |_| Paths::default(),
        |matches| Paths {
            config: matches.get_one::<PathBuf>("config").cloned(),
            db: matches.get_one::<PathBuf>("db").cloned(),
            models_dir: matches.get_one::<PathBuf>("models_dir").cloned(),
        },
    )
}

/// What `--version-info` prints, wherever the flag appears: the release and
/// the silver schemas it opens.
pub(super) fn version_info(arguments: &[OsString]) -> Option<Response> {
    arguments
        .iter()
        .skip(1)
        .any(|argument| argument == VERSION_INFO)
        .then_some(Response::Version {
            version: env!("CARGO_PKG_VERSION"),
            silver_schema: crate::storage::schema::SCHEMA_VERSION,
            silver_schema_min: crate::storage::schema::MIN_SCHEMA_VERSION,
            record_citations: ["message_id", "session_id"],
            session_citations: ["claude-code", "codex", "copilot"],
        })
}

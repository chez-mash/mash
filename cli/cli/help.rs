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

/// What a command line clap refused means: help or the version, which are
/// answers, or a usage error.
pub(super) fn parse_failure(
    error: &clap::Error,
    arguments: &[OsString],
) -> Result<Response, crate::AppError> {
    match error.kind() {
        // Bare `mash` is still a usage error, but it carries the help a
        // person needs rather than clap's one-line complaint.
        clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            let help = Cli::command().render_help().to_string();
            Err(crate::AppError::usage(help))
        }
        clap::error::ErrorKind::DisplayHelp => {
            let mut help = error.to_string();
            if let Some(notice) = crate::core::installation::help_notice(&help_paths(arguments)) {
                help.push('\n');
                help.push_str(notice);
                help.push('\n');
            }
            Ok(Response::Help { help })
        }
        // The short version is for a person, like help.
        clap::error::ErrorKind::DisplayVersion => Ok(Response::Help {
            help: concat!("mash ", env!("CARGO_PKG_VERSION")).to_owned(),
        }),
        _ => Err(crate::AppError::usage(error.to_string())),
    }
}

//! Uninstall guidance and its terminal presentation.
use std::fmt::Write as _;
use std::path::Path;

use super::{Binary, Content, Outcome, Request, Response, ServiceKind};
use crate::core::output::text::safe;

impl Response {
    #[must_use]
    pub fn terminal_summary(&self) -> String {
        let mut output = match (self.applied, self.incomplete()) {
            (false, _) => "Mash uninstall preview\n".to_owned(),
            (true, true) => "Mash uninstall needs attention\n".to_owned(),
            (true, false) => "Mash uninstall finished\n".to_owned(),
        };
        if self.services.is_empty() && self.agents.is_empty() && self.paths.is_empty() {
            output.push_str("\nNo Mash services, connections, or local data to remove.\n");
        } else {
            output.push('\n');
        }
        for service in &self.services {
            let label = match service.kind {
                ServiceKind::Qdrant => "the local search service",
                ServiceKind::Dashboard => "the remote dashboard service",
                ServiceKind::Watcher => "the history watcher service",
            };
            let name = safe(&service.name);
            report_step(&mut output, &format!("{label} ({name})"), &service.outcome);
        }
        for agent in &self.agents {
            let name = agent.agent.display_name();
            for (label, step) in [
                ("Mash search tools", &agent.mcp),
                ("the Mash plugin", &agent.plugin),
            ] {
                if let Some(step) = step {
                    report_step(&mut output, &format!("{label} from {name}"), &step.outcome);
                    if matches!(step.outcome, Outcome::Failed { .. }) {
                        let _ = writeln!(output, "  Command `{}`", step.command);
                    }
                }
            }
        }
        for item in &self.paths {
            let label = match item.content {
                Content::Configuration => "Mash settings",
                Content::Index => "Mash's copy of your chats",
                Content::Models => "downloaded models",
                Content::SearchData => "local search service data",
                Content::Tokens => "saved tokens",
                Content::Dashboard => "dashboard service settings and logs",
                Content::HookState => "agent hook state",
            };
            report_step(
                &mut output,
                &format!("{label} ({})", disk_size(item.bytes)),
                &item.outcome,
            );
            for path in &item.members {
                let _ = writeln!(output, "  {}", safe(&path.to_string_lossy()));
            }
        }

        output.push_str("\nKept\n");
        for item in &self.kept {
            if let Some(path) = &item.path {
                let _ = writeln!(
                    output,
                    "  {} ({})",
                    safe(&path.to_string_lossy()),
                    safe(&item.reason)
                );
            } else {
                let _ = writeln!(output, "  {}", safe(&item.reason));
            }
        }
        if !self.applied {
            output.push_str("  The Mash program itself. Remove it after uninstalling.\n");
        }

        output.push('\n');
        for hint in &self.next {
            let _ = writeln!(output, "{}", safe(hint));
        }
        output
    }
}

fn report_step(output: &mut String, label: &str, outcome: &Outcome) {
    let _ = match outcome {
        Outcome::Planned => writeln!(output, "Would remove {label}"),
        Outcome::Removed => writeln!(output, "Removed {label}"),
        Outcome::Failed { message } => {
            writeln!(output, "Failed to remove {label}\n  {}", safe(message))
        }
    };
}

fn disk_size(bytes: u64) -> String {
    for (unit, scale) in [
        ("TB", 1_000_000_000_000),
        ("GB", 1_000_000_000),
        ("MB", 1_000_000),
        ("kB", 1_000),
    ] {
        if bytes >= scale {
            return format!("{}.{} {unit}", bytes / scale, bytes % scale / (scale / 10));
        }
    }
    format!("{bytes} bytes")
}

/// How to remove the executable, judged from where it is installed.
pub(super) fn binary(executable: Option<&Path>) -> Binary {
    let Some(path) = executable else {
        return Binary {
            path: None,
            remove: "Remove the mash executable with whatever installed it.".to_owned(),
        };
    };
    let text = path.to_string_lossy().replace('\\', "/").to_lowercase();
    let remove = if ["/cellar/", "/homebrew/", "/linuxbrew/"]
        .iter()
        .any(|part| text.contains(part))
    {
        "brew uninstall mash".to_owned()
    } else if text.contains("/scoop/") {
        "scoop uninstall mash".to_owned()
    } else if text.contains("/.cargo/bin/") {
        "cargo uninstall mash".to_owned()
    } else if text == "/usr/bin/mash" {
        "sudo apt remove mash-cli".to_owned()
    } else {
        // The install script puts `mash-server` beside `mash`.
        let server = path.with_file_name(format!("mash-server{}", std::env::consts::EXE_SUFFIX));
        if server.is_file() {
            format!("Delete {} and {}", path.display(), server.display())
        } else {
            format!("Delete {}", path.display())
        }
    };
    Binary {
        path: Some(path.to_path_buf()),
        remove,
    }
}

pub(super) fn next(
    response: &Response,
    request: Request,
    custom_paths: bool,
    running: usize,
) -> Vec<String> {
    let command = if request.keep_history {
        "mash uninstall --yes --keep-history"
    } else {
        "mash uninstall --yes"
    };
    let mut next = Vec::new();
    if !request.apply {
        let confirmation = if custom_paths {
            "Add `--yes` to the command you just ran".to_owned()
        } else {
            format!("Run `{command}`")
        };
        next.push(format!(
            "Nothing was changed. {confirmation} to remove what is listed."
        ));
        if !request.keep_history
            && response
                .paths
                .iter()
                .any(|item| item.content == Content::Index)
        {
            next.push(if custom_paths {
                "To keep Mash's copy of your chats in place, preview again with `--keep-history`.".to_owned()
            } else {
                "To keep a copy of your indexed history first, run `mash export DIRECTORY`. To keep it in place, preview with `mash uninstall --keep-history`.".to_owned()
            });
        }
        return next;
    }
    let incomplete = response.incomplete();
    if incomplete {
        let retry = if custom_paths {
            "rerun the command with the same paths and options".to_owned()
        } else {
            format!("run `{command}` again")
        };
        next.push(format!(
            "Some steps failed and are marked `failed`. Fix what each message names, then {retry}."
        ));
    }
    if !response.agents.is_empty() {
        next.push("Restart your agents so they stop loading Mash.".to_owned());
    }
    if running > 0 {
        next.push(format!(
            "{running} Mash process(es) started before this ran, such as an agent's MCP server or a watcher in a terminal; each ends when the agent or terminal that started it closes."
        ));
    }
    let remove = &response.binary.remove;
    next.push(if incomplete {
        format!("After retrying the failed steps, remove the Mash program with this instruction: {remove}")
    } else {
        format!("Mash's executable is still installed. To remove it: {remove}")
    });
    next
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::{Connection, Item, Kept, Service, Step};
    use super::*;
    use crate::core::installation::Agent;

    fn uninstall_report(applied: bool) -> Response {
        let complete = if applied {
            Outcome::Removed
        } else {
            Outcome::Planned
        };
        let path = PathBuf::from("/example/history.sqlite3");
        let mut response = Response {
            applied,
            services: vec![Service {
                kind: ServiceKind::Qdrant,
                name: "mash-qdrant.service".to_owned(),
                outcome: complete.clone(),
            }],
            agents: vec![Connection {
                agent: Agent::Codex,
                mcp: None,
                plugin: Some(Step {
                    command: "codex plugin remove mash@mash",
                    outcome: if applied {
                        Outcome::Failed {
                            message: "offline\n\x1b[2J".to_owned(),
                        }
                    } else {
                        Outcome::Planned
                    },
                }),
            }],
            paths: vec![Item {
                content: Content::Index,
                path: path.clone(),
                members: vec![path, PathBuf::from("/example/history.sqlite3-wal")],
                bytes: 2_475_000,
                outcome: complete,
            }],
            kept: vec![Kept {
                path: Some(PathBuf::from("/example/backup")),
                reason: "Not created by Mash's local commands.".to_owned(),
            }],
            binary: Binary {
                path: None,
                remove: "brew uninstall mash".to_owned(),
            },
            next: Vec::new(),
        };
        response.next = next(
            &response,
            Request {
                apply: applied,
                keep_history: false,
            },
            true,
            0,
        );
        response
    }

    #[test]
    fn uninstall_terminal_preview_lists_files_and_preserves_confirmation_flags() {
        let preview = uninstall_report(false).terminal_summary();
        for expected in [
            "Mash uninstall preview",
            "Would remove the local search service (mash-qdrant.service)",
            "Would remove the Mash plugin from Codex",
            "Would remove Mash's copy of your chats (2.4 MB)",
            "  /example/history.sqlite3-wal",
            "  /example/backup (Not created by Mash's local commands.)",
            "Nothing was changed. Add `--yes` to the command you just ran",
        ] {
            assert!(preview.contains(expected), "{preview}");
        }
    }

    #[test]
    fn uninstall_terminal_names_failed_steps_and_escapes_process_output() {
        let report = uninstall_report(true);
        let summary = report.terminal_summary();
        assert!(report.incomplete());
        for expected in [
            "Mash uninstall needs attention",
            "Removed the local search service (mash-qdrant.service)",
            "Failed to remove the Mash plugin from Codex",
            "offline\\n",
            "codex plugin remove mash@mash",
            "rerun the command with the same paths and options",
            "After retrying the failed steps, remove the Mash program",
        ] {
            assert!(summary.contains(expected), "{summary}");
        }
        assert!(!summary.contains('\x1b'), "{summary}");
    }
}

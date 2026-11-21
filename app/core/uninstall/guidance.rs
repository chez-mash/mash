//! Instructions for confirming or retrying uninstall and removing the executable.
use std::path::Path;

use super::{Binary, Content, Request, Response};

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
    if response.incomplete() {
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
    next.push(format!(
        "Mash's executable is still installed. To remove it: {}",
        response.binary.remove
    ));
    next
}

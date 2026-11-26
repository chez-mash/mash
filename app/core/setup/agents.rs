//! Finding and running the agent CLIs (`claude`, `codex`, `copilot`) that
//! `mash init` registers Mash with.
//!
//! A terminal has the user's full PATH, but a dashboard started by the
//! desktop app, launchd or systemd gets a minimal one, so looking on the
//! process PATH alone misses agents a person can run in their shell. A
//! program is therefore looked for, in order, on the process PATH, on the
//! PATH the user's login shell sets up, and in the folders the agents'
//! installers and the usual package managers put commands in. The same
//! search becomes the PATH an agent CLI runs with, because several are
//! Node scripts that need `node` from the same places. Nothing here guesses
//! beyond those folders: a program found nowhere is reported as missing.
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Duration;

const AGENT_TIMEOUT: Duration = Duration::from_secs(120);
/// How long the login shell may take to report its PATH.
#[cfg(unix)]
const SHELL_TIMEOUT: Duration = Duration::from_secs(5);
#[cfg(unix)]
const MARKER: &str = "__MASH_LOGIN_PATH__=";

/// Where a command named `program` is, searched the way a person's shell would.
pub fn which(program: &str) -> Option<PathBuf> {
    locate(program, None)
}

/// Every `program` in this process's PATH, in the order a lookup tries them,
/// each file once (a link and the file it names are one).
pub fn on_path(program: &str) -> Vec<PathBuf> {
    let (Ok(base), Some(path)) = (std::env::current_dir(), std::env::var_os("PATH")) else {
        return Vec::new();
    };
    let mut found: Vec<(PathBuf, PathBuf)> = Vec::new();
    for candidate in crate::platform::find_programs(program, &path, &base) {
        let identity = std::fs::canonicalize(&candidate).unwrap_or_else(|_| candidate.clone());
        if !found.iter().any(|(_, seen)| *seen == identity) {
            found.push((candidate, identity));
        }
    }
    found.into_iter().map(|(candidate, _)| candidate).collect()
}

/// [`which`], or only `only` when given (a test's stand-in for the user's
/// search path).
pub(crate) fn locate(program: &str, only: Option<&OsStr>) -> Option<PathBuf> {
    let base = std::env::current_dir().ok()?;
    if let Some(only) = only {
        return crate::platform::find_program(program, only, &base);
    }
    let process = std::env::var_os("PATH").unwrap_or_default();
    if let Some(found) = crate::platform::find_program(program, &process, &base) {
        return Some(found);
    }
    let search = search_path(Some(&process), home().as_deref(), login_path());
    crate::platform::find_program(program, &search, &base)
}

/// [`locate`] off the async runtime: asking the login shell can take seconds.
pub(super) async fn find(program: &'static str, only: Option<&OsStr>) -> Option<PathBuf> {
    let only = only.map(OsStr::to_os_string);
    crate::blocking(move || Ok(locate(program, only.as_deref())))
        .await
        .ok()
        .flatten()
}

/// The PATH an agent CLI found at `program` runs with: its own folder first,
/// then the same places [`which`] looks.
fn child_path(program: &Path) -> OsString {
    let process = std::env::var_os("PATH").unwrap_or_default();
    // The login shell is asked only when the process PATH missed something.
    let login = LOGIN.get().cloned().flatten();
    let search = search_path(Some(&process), home().as_deref(), login);
    let own = program.parent().map(Path::to_path_buf);
    let mut directories: Vec<PathBuf> = own.into_iter().collect();
    directories.extend(std::env::split_paths(&search));
    joined(directories)
}

fn home() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf())
}

/// The process PATH, then the login shell's, then the usual install folders,
/// each folder once.
fn search_path(process: Option<&OsStr>, home: Option<&Path>, login: Option<OsString>) -> OsString {
    let mut directories: Vec<PathBuf> = Vec::new();
    for path in [process.map(OsStr::to_os_string), login]
        .into_iter()
        .flatten()
    {
        directories.extend(std::env::split_paths(&path));
    }
    directories.extend(usual_folders(home));
    joined(directories)
}

fn joined(directories: Vec<PathBuf>) -> OsString {
    let mut unique: Vec<PathBuf> = Vec::new();
    for directory in directories {
        if !directory.as_os_str().is_empty() && !unique.contains(&directory) {
            unique.push(directory);
        }
    }
    std::env::join_paths(unique).unwrap_or_default()
}

/// Folders the agents' own installers and the common package managers put
/// commands in, for an account whose PATH does not name them.
fn usual_folders(home: Option<&Path>) -> Vec<PathBuf> {
    let mut folders = Vec::new();
    if let Some(home) = home {
        for relative in [
            ".local/bin",
            ".claude/local",
            ".npm-global/bin",
            ".bun/bin",
            ".volta/bin",
            ".yarn/bin",
            ".local/share/pnpm",
            "Library/pnpm",
            ".nix-profile/bin",
        ] {
            folders.push(home.join(relative));
        }
        // nvm keeps one folder per Node version; prefer the newest name.
        let mut versions: Vec<PathBuf> = std::fs::read_dir(home.join(".nvm/versions/node"))
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path().join("bin"))
            .collect();
        versions.sort();
        folders.extend(versions.into_iter().rev());
    }
    if cfg!(windows) {
        for (variable, relative) in [("APPDATA", "npm"), ("LOCALAPPDATA", "pnpm")] {
            if let Some(root) = std::env::var_os(variable) {
                folders.push(PathBuf::from(root).join(relative));
            }
        }
    } else {
        folders
            .extend(["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"].map(PathBuf::from));
    }
    folders
}

static LOGIN: std::sync::OnceLock<Option<OsString>> = std::sync::OnceLock::new();

/// The PATH the user's login shell sets up, asked once per process.
fn login_path() -> Option<OsString> {
    LOGIN.get_or_init(ask_login_shell).clone()
}

#[cfg(unix)]
fn ask_login_shell() -> Option<OsString> {
    let shell = std::env::var_os("SHELL")
        .filter(|shell| !shell.is_empty())
        .unwrap_or_else(|| {
            if cfg!(target_os = "macos") {
                "/bin/zsh"
            } else {
                "/bin/sh"
            }
            .into()
        });
    // Interactive as well as login: many people set PATH in .zshrc or .bashrc.
    // The marker separates the answer from anything their startup files print.
    let mut child = std::process::Command::new(shell)
        .args([
            "-l",
            "-i",
            "-c",
            &format!("printf '\\n{MARKER}%s\\n' \"$PATH\""),
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let started = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() < SHELL_TIMEOUT => {
                std::thread::sleep(Duration::from_millis(25));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
    let mut output = String::new();
    std::io::Read::read_to_string(&mut child.stdout.take()?, &mut output).ok()?;
    parse_login_output(&output)
}

#[cfg(not(unix))]
fn ask_login_shell() -> Option<OsString> {
    None
}

#[cfg(unix)]
fn parse_login_output(output: &str) -> Option<OsString> {
    output
        .lines()
        .rev()
        .find_map(|line| line.strip_prefix(MARKER))
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(OsString::from)
}

pub(super) fn display(program: &Path, arguments: &[OsString]) -> String {
    std::iter::once(program.display().to_string())
        .chain(arguments.iter().map(|a| a.to_string_lossy().into_owned()))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) async fn step(program: &Path, arguments: &[OsString]) -> super::Step {
    let command = display(program, arguments);
    match run(program, arguments).await {
        Ok(()) => super::Step::Done { command },
        Err(message) => super::Step::Failed { command, message },
    }
}

/// Run an agent CLI with the search path as its PATH.
pub(crate) async fn run(program: &Path, arguments: &[impl AsRef<OsStr>]) -> Result<(), String> {
    use crate::service::Failure;
    let path = child_path(program);
    match crate::service::output_with_path(program, arguments, AGENT_TIMEOUT, &path).await {
        Err(Failure::TimedOut { after, .. }) => {
            Err(format!("timed out after {} seconds", after.as_secs()))
        }
        Err(Failure::Unavailable { reason, .. }) => Err(reason),
        Err(other) => Err(other.describe()),
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stderr);
            let text = if text.trim().is_empty() {
                String::from_utf8_lossy(&output.stdout)
            } else {
                text
            };
            Err(text.trim().chars().take(500).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn executable(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    #[test]
    fn agents_are_found_in_their_install_folders_without_a_path() {
        let home = tempfile::tempdir().unwrap();
        let name = if cfg!(windows) {
            "claude.exe"
        } else {
            "claude"
        };
        executable(&home.path().join(".local/bin").join(name));
        executable(
            &home
                .path()
                .join(".nvm/versions/node/v22.1.0/bin")
                .join(name),
        );
        let base = std::env::current_dir().unwrap();
        // A dashboard under launchd or systemd: no PATH, or a minimal one.
        for process in [None, Some(OsStr::new("/nonexistent"))] {
            let search = search_path(process, Some(home.path()), None);
            assert!(crate::platform::find_program("claude", &OsString::new(), &base).is_none());
            assert_eq!(
                crate::platform::find_program("claude", &search, &base),
                Some(home.path().join(".local/bin").join(name)),
            );
        }
        // The process PATH comes first, then the login shell's.
        let login = home.path().join("login");
        executable(&login.join(name));
        let search = search_path(None, Some(home.path()), Some(login.clone().into()));
        assert_eq!(
            crate::platform::find_program("claude", &search, &base),
            Some(login.join(name))
        );
    }

    #[test]
    fn every_copy_is_found_in_the_order_a_lookup_tries_them() {
        let root = tempfile::tempdir().unwrap();
        let name = if cfg!(windows) { "mash.exe" } else { "mash" };
        let (first, empty, second) = (
            root.path().join("first"),
            root.path().join("empty"),
            root.path().join("second"),
        );
        executable(&first.join(name));
        std::fs::create_dir_all(&empty).unwrap();
        executable(&second.join(name));
        let path = std::env::join_paths([&first, &empty, &second]).unwrap();
        let base = std::env::current_dir().unwrap();
        let found: Vec<PathBuf> = crate::platform::find_programs("mash", &path, &base).collect();
        assert_eq!(found, [first.join(name), second.join(name)]);
        assert_eq!(
            crate::platform::find_program("mash", &path, &base),
            Some(first.join(name))
        );
    }

    #[test]
    #[cfg(unix)]
    fn login_shell_path_is_read_past_whatever_its_startup_files_print() {
        let output = format!("Welcome back!\n\n{MARKER}/opt/homebrew/bin:/usr/bin\n");
        assert_eq!(
            parse_login_output(&output),
            Some(OsString::from("/opt/homebrew/bin:/usr/bin"))
        );
        assert_eq!(parse_login_output("no marker here\n"), None);
        assert_eq!(parse_login_output(&format!("{MARKER}\n")), None);
    }

    #[test]
    fn search_path_names_each_folder_once() {
        let search = search_path(
            Some(OsStr::new("/usr/bin")),
            None,
            Some(OsString::from("/usr/bin")),
        );
        let folders: Vec<PathBuf> = std::env::split_paths(&search).collect();
        let usr = folders
            .iter()
            .filter(|folder| folder.as_path() == Path::new("/usr/bin"));
        assert_eq!(usr.count(), 1);
    }
}

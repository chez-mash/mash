//! The `mash` command the Windows app brings with it: the parts that are
//! plain text and files, kept apart from the registry so they are tested on
//! every system.
//!
//! The command is a copy of the app's `mash.exe` in a `bin` folder of its
//! own inside the install folder, and that folder is appended to the user's
//! PATH. A real `.exe` is what a program started without a shell finds
//! (`CreateProcess` adds only `.exe` to a bare name, so a `.cmd` would be
//! invisible to an agent that starts `mash mcp` directly), and a folder of
//! its own keeps the app, its uninstaller and anything else in the install
//! folder off PATH. It is a copy and not a hard link so that a `mash mcp`
//! left running from the command holds only the copy open, never the file
//! an installer has to replace. Appended, never put first: a `mash` that
//! was already on PATH keeps winning, so nothing is shadowed.

use std::path::{Path, PathBuf};

/// The app's identifier, which names its folder under `%APPDATA%`.
pub const IDENTIFIER: &str = "io.getmash.desktop";
/// The folder inside the install folder that goes on PATH.
pub const FOLDER: &str = "bin";
/// The argument by which the installer asks the app to act (hooks.nsh).
const REQUEST: &str = "--command-line-tool";
/// What a name on PATH may end in and still be run as `mash`.
const RUNNABLE: [&str; 5] = ["exe", "cmd", "bat", "com", "ps1"];
/// The start of the name a copy still running is moved to, out of the way.
const ASIDE: &str = "mash.old-";
/// The names a long-lived `mash` (such as `mash mcp`) moves its own program
/// to while it runs, so that its installed path can be replaced
/// (`release_installed_executable` in the engine).
const RUNNING: [&str; 2] = [".mash-running-", ".mash-staged-"];

/// What the installer asks of the app.
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    /// The app has just been installed or installed over: put the command
    /// in place, unless the person removed it.
    Installed,
    /// The app is about to be uninstalled: take the command away.
    Uninstalling,
}

/// The installer's request among the app's arguments, if that is what they
/// are. Anything else (no arguments, a mash:// link) starts the app.
pub fn request(mut arguments: impl Iterator<Item = String>) -> Option<Request> {
    if arguments.next()? != REQUEST {
        return None;
    }
    match arguments.next()?.as_str() {
        "installed" => Some(Request::Installed),
        "uninstalling" => Some(Request::Uninstalling),
        _ => None,
    }
}

/// A PATH entry as it is compared: Windows ignores case, a trailing
/// separator and quotes around an entry.
fn folded(entry: &str) -> String {
    entry
        .trim()
        .trim_matches('"')
        .trim_end_matches(['\\', '/'])
        .to_lowercase()
}

/// Whether `path` (a PATH value) already has `folder`.
pub fn has_folder(path: &str, folder: &str) -> bool {
    let folder = folded(folder);
    path.split(';').any(|entry| folded(entry) == folder)
}

/// `path` with `folder` added at the end, or nothing when it is already
/// there. Every other entry is kept exactly as it was written.
pub fn with_folder(path: &str, folder: &str) -> Option<String> {
    if has_folder(path, folder) {
        return None;
    }
    let kept = path.trim_end_matches(';');
    Some(if kept.is_empty() {
        folder.to_owned()
    } else {
        format!("{kept};{folder}")
    })
}

/// `path` without `folder`, or nothing when it is not there. Every other
/// entry is kept exactly as it was written.
pub fn without_folder(path: &str, folder: &str) -> Option<String> {
    if !has_folder(path, folder) {
        return None;
    }
    let folder = folded(folder);
    Some(
        path.split(';')
            .filter(|entry| folded(entry) != folder)
            .collect::<Vec<_>>()
            .join(";"),
    )
}

/// `text` with each `%NAME%` that `variable` knows replaced by its value,
/// as Windows expands a PATH kept in the registry.
pub fn expand(text: &str, variable: impl Fn(&str) -> Option<String>) -> String {
    let mut expanded = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('%') {
        expanded.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after
            .find('%')
            .and_then(|end| variable(&after[..end]).map(|value| (end, value)))
        {
            Some((end, value)) => {
                expanded.push_str(&value);
                rest = &after[end + 1..];
            }
            None => {
                expanded.push('%');
                rest = after;
            }
        }
    }
    expanded.push_str(rest);
    expanded
}

/// The folders of a PATH value, in order, with their variables expanded.
pub fn folders(path: &str, variable: impl Fn(&str) -> Option<String>) -> Vec<PathBuf> {
    path.split(';')
        .map(|entry| expand(entry.trim().trim_matches('"'), &variable))
        .filter(|entry| !entry.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// Another `mash` that a terminal finds before the app's own: the first one
/// in `folders` (the system's PATH, then the user's) ahead of `ours`. The
/// app's folder goes last, so anything found when it is not yet listed is
/// ahead of it too.
pub fn other_before(folders: &[PathBuf], ours: &Path) -> Option<PathBuf> {
    let ours = folded(&ours.to_string_lossy());
    for folder in folders {
        if folded(&folder.to_string_lossy()) == ours {
            return None;
        }
        if !folder.is_absolute() {
            continue;
        }
        let found = RUNNABLE
            .iter()
            .map(|kind| folder.join(format!("mash.{kind}")))
            .find(|file| file.is_file());
        if found.is_some() {
            return found;
        }
    }
    None
}

/// Whether the command is the app's present `mash`: the same size and the
/// same time of writing, which `place` carries over.
pub fn in_step(engine: &Path, command: &Path) -> bool {
    let (Ok(engine), Ok(command)) = (std::fs::metadata(engine), std::fs::metadata(command)) else {
        return false;
    };
    engine.len() == command.len()
        && matches!((engine.modified(), command.modified()), (Ok(one), Ok(other)) if one == other)
}

/// Make `command` a copy of `engine`. A copy that is still running, such as
/// an agent's `mash mcp`, cannot be written over or deleted, but it can be
/// renamed: it is moved aside and goes the next time nothing runs it.
pub fn place(engine: &Path, command: &Path) -> std::io::Result<()> {
    let folder = command.parent().unwrap_or(command);
    std::fs::create_dir_all(folder)?;
    let draft = folder.join("mash.new");
    std::fs::copy(engine, &draft)?;
    std::fs::File::options()
        .write(true)
        .open(&draft)?
        .set_modified(std::fs::metadata(engine)?.modified()?)?;
    set_aside(command)?;
    std::fs::rename(&draft, command)?;
    sweep(folder);
    Ok(())
}

/// Take the command away, and its folder when nothing is left in it.
pub fn remove(command: &Path) -> std::io::Result<()> {
    let folder = command.parent().unwrap_or(command);
    set_aside(command)?;
    let _ = std::fs::remove_file(folder.join("mash.new"));
    sweep(folder);
    // A copy still running stays, and so does the folder, until it ends.
    let _ = std::fs::remove_dir(folder);
    Ok(())
}

/// Delete `command`, or move it aside when it is running.
fn set_aside(command: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(command) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => {
            let folder = command.parent().unwrap_or(command);
            let aside = (0..1000)
                .map(|number| folder.join(format!("{ASIDE}{number}")))
                .find(|aside| !aside.exists())
                .ok_or_else(|| std::io::Error::other("too many copies moved aside"))?;
            std::fs::rename(command, aside)
        }
    }
}

/// Delete the copies moved aside earlier, by this app or by `mash` itself,
/// that nothing runs any more; one still running cannot be deleted and stays.
fn sweep(folder: &Path) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(ASIDE) || RUNNING.iter().any(|start| name.starts_with(start)) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// The note that another `mash` comes first, in the words the Mac app uses
/// for a separate copy in `~/.local/bin`.
fn separate(other: &Path) -> String {
    format!(
        "\n\n{} is already a separate copy of mash, such as one winget, Scoop or the install \
         script put there, and it comes first on your PATH, so mash in a terminal runs that \
         copy. Mash left it as it is; it updates separately from this app. To use this app’s \
         mash instead, remove that copy.",
        other.display()
    )
}

/// What to say once the command is in place.
pub fn installed(command: &Path, other: Option<&Path>) -> String {
    format!(
        "{} now runs this app’s mash and updates with the app. Its folder is on your PATH. \
         Open a new terminal and run mash. To remove it, choose Install Command-Line Tool… \
         again.{}",
        command.display(),
        other.map_or_else(String::new, separate)
    )
}

/// What to ask when the command is already in place.
pub fn offer(command: &Path, other: Option<&Path>) -> String {
    format!(
        "{} already runs this app’s mash and updates with the app, and its folder is on your \
         PATH. Remove it?{}",
        command.display(),
        other.map_or_else(String::new, separate)
    )
}

/// What to say once the command has been taken away.
pub fn removed(command: &Path) -> String {
    format!(
        "Removed {} and took its folder off your PATH. Installing a newer version of the app \
         leaves it off; choose Install Command-Line Tool… to put it back.",
        command.display()
    )
}

#[cfg(test)]
mod tests {
    use super::{
        FOLDER, IDENTIFIER, Request, expand, folders, has_folder, in_step, installed, offer,
        other_before, place, remove, removed, request, with_folder, without_folder,
    };
    use std::path::{Path, PathBuf};

    const OURS: &str = r"C:\Users\sam\AppData\Local\Mash\bin";

    fn arguments(words: &[&str]) -> impl Iterator<Item = String> {
        words
            .iter()
            .map(|word| (*word).to_owned())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn only_the_installers_words_are_a_request() {
        assert_eq!(
            request(arguments(&["--command-line-tool", "installed"])),
            Some(Request::Installed)
        );
        assert_eq!(
            request(arguments(&["--command-line-tool", "uninstalling"])),
            Some(Request::Uninstalling)
        );
        assert_eq!(request(arguments(&[])), None);
        assert_eq!(request(arguments(&["mash://chat/abc"])), None);
        assert_eq!(request(arguments(&["--command-line-tool"])), None);
        assert_eq!(request(arguments(&["--command-line-tool", "other"])), None);
        assert_eq!(
            request(arguments(&["installed", "--command-line-tool"])),
            None
        );
    }

    #[test]
    fn the_installer_and_the_app_agree() {
        // The hooks name the request, and the configuration the hooks and
        // the identifier this module restates.
        let hooks = include_str!("../../windows/hooks.nsh");
        assert!(hooks.contains("--command-line-tool installed"), "{hooks}");
        assert!(
            hooks.contains(r#""QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'"#),
            "{hooks}"
        );
        assert!(
            hooks.contains("--command-line-tool uninstalling"),
            "{hooks}"
        );
        let configuration: serde_json::Value =
            serde_json::from_str(include_str!("../../tauri.conf.json")).unwrap();
        assert_eq!(configuration["identifier"], IDENTIFIER);
        assert_eq!(
            configuration["bundle"]["windows"]["nsis"]["installerHooks"],
            "./windows/hooks.nsh"
        );
        assert_eq!(FOLDER, "bin");
    }

    #[test]
    fn the_folder_goes_last_and_nothing_else_changes() {
        let path = r"%USERPROFILE%\.cargo\bin;C:\Tools\;;C:\Program Files\Git\cmd";
        assert_eq!(
            with_folder(path, OURS).unwrap(),
            format!("{path};{OURS}"),
            "appended, so a mash that was already found still is"
        );
        assert_eq!(with_folder("", OURS).unwrap(), OURS);
        assert_eq!(
            with_folder(r"C:\Tools;", OURS).unwrap(),
            format!(r"C:\Tools;{OURS}")
        );
        // Already there, however it was written.
        for written in [
            OURS.to_owned(),
            format!(r"{OURS}\"),
            OURS.to_uppercase(),
            format!("\"{OURS}\""),
            format!(" {OURS} "),
        ] {
            let path = format!(r"C:\Tools;{written};C:\Other");
            assert!(has_folder(&path, OURS), "{path}");
            assert_eq!(with_folder(&path, OURS), None, "{path}");
            assert_eq!(
                without_folder(&path, OURS).unwrap(),
                r"C:\Tools;C:\Other",
                "{path}"
            );
        }
        // A folder that only starts the same way is another folder.
        assert!(!has_folder(&format!("{OURS}2"), OURS));
    }

    #[test]
    fn removing_the_folder_keeps_every_other_entry_as_written() {
        let before = r"%USERPROFILE%\.cargo\bin;C:\Tools\;;C:\Program Files\Git\cmd";
        let added = with_folder(before, OURS).unwrap();
        assert_eq!(without_folder(&added, OURS).unwrap(), before);
        assert_eq!(without_folder(before, OURS), None, "not there: no write");
        assert_eq!(without_folder(OURS, OURS).unwrap(), "");
        let twice = format!(r"{OURS};C:\Tools;{OURS}\");
        assert_eq!(without_folder(&twice, OURS).unwrap(), r"C:\Tools");
    }

    #[test]
    fn expands_the_variables_it_knows() {
        let variable = |name: &str| {
            name.eq_ignore_ascii_case("userprofile")
                .then(|| r"C:\Users\sam".to_owned())
        };
        assert_eq!(
            expand(r"%USERPROFILE%\.cargo\bin", variable),
            r"C:\Users\sam\.cargo\bin"
        );
        assert_eq!(expand(r"%NOPE%\bin", variable), r"%NOPE%\bin");
        assert_eq!(
            expand("50% of %USERPROFILE%", variable),
            r"50% of C:\Users\sam"
        );
        assert_eq!(expand("plain", variable), "plain");
        assert_eq!(
            folders(r#" "%USERPROFILE%\a b" ;;C:\x"#, variable),
            [PathBuf::from(r"C:\Users\sam\a b"), PathBuf::from(r"C:\x")]
        );
    }

    fn scratch(name: &str) -> PathBuf {
        let folder =
            std::env::temp_dir().join(format!("mash command {name} {}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    #[test]
    fn finds_another_mash_only_ahead_of_the_apps_own() {
        let root = scratch("other");
        let (empty, winget, ours, later) = (
            root.join("empty"),
            root.join("WinGet").join("Links"),
            root.join("Mash").join("bin"),
            root.join("later"),
        );
        for folder in [&empty, &winget, &ours, &later] {
            std::fs::create_dir_all(folder).unwrap();
        }
        std::fs::write(ours.join("mash.exe"), "ours").unwrap();
        std::fs::write(later.join("mash.exe"), "after ours").unwrap();
        // A folder called mash, and a name that only starts with it.
        std::fs::create_dir_all(empty.join("mash.exe")).unwrap();
        std::fs::write(empty.join("mash-server.exe"), "another program").unwrap();
        let listed = [
            PathBuf::from("relative"),
            empty.clone(),
            winget.clone(),
            ours.clone(),
            later.clone(),
        ];
        assert_eq!(other_before(&listed, &ours), None, "nothing ahead of ours");
        std::fs::write(winget.join("mash.cmd"), "a script").unwrap();
        assert_eq!(other_before(&listed, &ours), Some(winget.join("mash.cmd")));
        std::fs::write(winget.join("mash.exe"), "a program").unwrap();
        assert_eq!(other_before(&listed, &ours), Some(winget.join("mash.exe")));
        // Not yet on PATH: the folder will go last, so every copy is ahead.
        assert_eq!(
            other_before(&[empty, later.clone()], &ours),
            Some(later.join("mash.exe"))
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_command_follows_the_apps_mash() {
        let root = scratch("place");
        let engine = root.join("mash.exe");
        let command = root.join("bin").join("mash.exe");
        std::fs::write(&engine, "version one").unwrap();
        assert!(!in_step(&engine, &command), "nothing there yet");
        place(&engine, &command).unwrap();
        assert_eq!(std::fs::read_to_string(&command).unwrap(), "version one");
        assert!(in_step(&engine, &command));
        // An update replaces the app's mash; the command is then behind.
        std::fs::write(&engine, "version two!").unwrap();
        assert!(!in_step(&engine, &command));
        // A copy moved aside earlier goes once nothing runs it.
        std::fs::write(root.join("bin").join("mash.old-0"), "was running").unwrap();
        std::fs::write(root.join("bin").join(".mash-running-42.exe"), "ran mcp").unwrap();
        place(&engine, &command).unwrap();
        assert_eq!(std::fs::read_to_string(&command).unwrap(), "version two!");
        assert!(in_step(&engine, &command));
        let left: Vec<_> = std::fs::read_dir(root.join("bin"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(left, ["mash.exe"], "no draft and nothing moved aside");
        remove(&command).unwrap();
        assert!(!root.join("bin").exists(), "the folder goes with it");
        remove(&command).unwrap();
        assert!(engine.exists(), "the app's own mash is never touched");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn says_what_the_mac_app_says_about_a_separate_copy() {
        let command = Path::new(r"C:\Mash\bin\mash.exe");
        let other = Path::new(r"C:\WinGet\Links\mash.exe");
        let alone = installed(command, None);
        assert!(alone.starts_with(r"C:\Mash\bin\mash.exe now runs this app’s mash"));
        assert!(alone.contains("Open a new terminal and run mash."));
        assert!(alone.ends_with("choose Install Command-Line Tool… again."));
        for said in [installed(command, Some(other)), offer(command, Some(other))] {
            assert!(
                said.contains(r"C:\WinGet\Links\mash.exe is already a separate copy of mash"),
                "{said}"
            );
            assert!(
                said.contains("Mash left it as it is; it updates separately from this app."),
                "{said}"
            );
            assert!(said.contains("To use this app’s mash instead, remove that"));
        }
        assert!(offer(command, None).ends_with("Remove it?"));
        assert!(removed(command).starts_with(r"Removed C:\Mash\bin\mash.exe"));
    }
}

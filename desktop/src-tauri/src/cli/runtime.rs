//! The Visual C++ runtime beside the Windows command. `mash.exe` needs
//! `VCRUNTIME140.dll`, `MSVCP140.dll` and their companions, which Windows
//! does not include, and looks for them beside itself first. The installer
//! puts them beside the app's `mash.exe`; the command is a copy of that
//! program in a folder of its own, so it needs its own copies of them.

use std::path::{Path, PathBuf};

/// The libraries beside `engine`.
fn libraries(engine: &Path) -> Vec<PathBuf> {
    let Some(entries) = engine.parent().and_then(|folder| folder.read_dir().ok()) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("dll"))
        })
        .collect()
}

/// Copy the libraries beside `engine` to beside `command`, where they are
/// not there already. One a running command holds open is in place, and
/// stays as it is.
pub fn carry(engine: &Path, command: &Path) -> std::io::Result<()> {
    let folder = command.parent().unwrap_or(command);
    for library in libraries(engine) {
        let Some(name) = library.file_name() else {
            continue;
        };
        let copy = folder.join(name);
        let same = matches!(
            (std::fs::metadata(&library), std::fs::metadata(&copy)),
            (Ok(one), Ok(other)) if one.len() == other.len()
        );
        if same {
            continue;
        }
        if let Err(error) = std::fs::copy(&library, &copy)
            && !copy.is_file()
        {
            return Err(error);
        }
    }
    Ok(())
}

/// Take the libraries beside `command` away, before its folder goes.
pub fn drop(command: &Path) {
    for library in libraries(command) {
        let _ = std::fs::remove_file(library);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_command_gets_the_engines_libraries_and_loses_them_with_it() {
        let root =
            std::env::temp_dir().join(format!("mash command runtime {}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("bin")).unwrap();
        let engine = root.join("mash.exe");
        let command = root.join("bin").join("mash.exe");
        for (name, text) in [
            ("mash.exe", "engine"),
            ("vcruntime140.dll", "runtime"),
            ("MSVCP140.DLL", "c++"),
            ("uninstall.exe", "not a library"),
        ] {
            std::fs::write(root.join(name), text).unwrap();
        }
        std::fs::write(&command, "engine").unwrap();
        carry(&engine, &command).unwrap();
        let mut copied: Vec<_> = std::fs::read_dir(root.join("bin"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        copied.sort();
        assert_eq!(copied, ["MSVCP140.DLL", "mash.exe", "vcruntime140.dll"]);
        // A newer runtime replaces the copy; an unchanged one is left alone.
        std::fs::write(root.join("vcruntime140.dll"), "newer runtime").unwrap();
        carry(&engine, &command).unwrap();
        assert_eq!(
            std::fs::read_to_string(root.join("bin").join("vcruntime140.dll")).unwrap(),
            "newer runtime"
        );
        drop(&command);
        assert_eq!(std::fs::read_dir(root.join("bin")).unwrap().count(), 1);
        assert!(
            root.join("vcruntime140.dll").exists(),
            "the app keeps its own"
        );
        std::fs::remove_dir_all(&root).unwrap();
    }
}

//! Asking a `mash` process to stop, on each platform.
//!
//! `mash dash` finishes its open requests and closes its event streams when it
//! is interrupted: SIGTERM or SIGINT on macOS and Linux, Ctrl+C on Windows.
//! Forcing a process to end is the last resort, after it has had time to
//! stop on its own.

/// Ask process `pid` to stop.
#[cfg(unix)]
pub fn interrupt(pid: u32) {
    if let Ok(pid) = libc::pid_t::try_from(pid) {
        // SAFETY: kill only sends a signal; a stale number fails harmlessly.
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }
    }
}

/// End process `pid` at once.
#[cfg(unix)]
pub fn force(pid: u32) {
    if let Ok(pid) = libc::pid_t::try_from(pid) {
        // SAFETY: as above.
        unsafe {
            libc::kill(pid, libc::SIGKILL);
        }
    }
}

/// Whether process `pid` is running a program called `mash`.
#[cfg(unix)]
pub fn is_mash(pid: u32) -> bool {
    std::process::Command::new("ps")
        .args(["-o", "comm=", "-p", &pid.to_string()])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|output| output.status.success())
        .is_some_and(|output| names_mash(&String::from_utf8_lossy(&output.stdout)))
}

/// Ask process `pid` to stop by sending Ctrl+C to its console, the one signal
/// a Windows console program can handle. The app has no console of its own,
/// so it borrows the engine's for a moment, ignoring the Ctrl+C itself.
#[cfg(windows)]
pub fn interrupt(pid: u32) {
    use windows_sys::Win32::System::Console::{
        AttachConsole, CTRL_C_EVENT, FreeConsole, GenerateConsoleCtrlEvent, SetConsoleCtrlHandler,
    };
    // SAFETY: these calls take plain values and only change which console
    // this process is attached to, which it gives back before returning.
    unsafe {
        if AttachConsole(pid) == 0 {
            return;
        }
        SetConsoleCtrlHandler(None, 1);
        GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0);
        FreeConsole();
        // The event arrives asynchronously; keep ignoring it until it has.
        std::thread::sleep(std::time::Duration::from_millis(100));
        SetConsoleCtrlHandler(None, 0);
    }
}

/// End process `pid` at once.
#[cfg(windows)]
pub fn force(pid: u32) {
    let _ = windows_command("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .status();
}

/// Whether process `pid` is running a program called `mash`.
#[cfg(windows)]
pub fn is_mash(pid: u32) -> bool {
    windows_command("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output()
        .ok()
        .is_some_and(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line.split(',').next().is_some_and(names_mash))
        })
}

/// A command that opens no console window.
#[cfg(windows)]
fn windows_command(program: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    let mut command = std::process::Command::new(program);
    command
        .creation_flags(0x0800_0000)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    command
}

/// Whether a process name from `ps` or `tasklist` is the `mash` program.
fn names_mash(name: &str) -> bool {
    let name = name.trim().trim_matches('"');
    let file = name.rsplit(['/', '\\']).next().unwrap_or(name);
    file.eq_ignore_ascii_case("mash") || file.eq_ignore_ascii_case("mash.exe")
}

#[cfg(test)]
mod tests {
    use super::names_mash;

    #[test]
    fn recognises_the_mash_program_by_name() {
        for name in [
            "mash\n",
            "/Applications/Mash.app/Contents/MacOS/mash",
            "\"mash.exe\"",
            "C:\\Program Files\\Mash\\mash.exe",
        ] {
            assert!(names_mash(name), "{name:?}");
        }
        for name in ["", "mash-server", "smash", "/usr/bin/mashy", "\"node.exe\""] {
            assert!(!names_mash(name), "{name:?}");
        }
    }
}

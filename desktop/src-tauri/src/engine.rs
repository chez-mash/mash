//! The Mash engine behind the window: the `mash` binary that ships inside the
//! app, running `mash dash` on a private loopback port. The window shows the
//! same dashboard a browser does, served by the same code; the app adds no
//! second implementation of anything the dashboard does.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// How long `mash dash` may take to say where it is listening.
const STARTUP: Duration = Duration::from_secs(60);

/// The port the app asks for first. The dashboard keeps the reader's theme and
/// layout in browser storage, which belongs to one address, so a port that
/// stays the same keeps those choices from one start to the next. When the
/// port is taken the app uses any free one and the choices start fresh.
const PREFERRED: &str = "127.0.0.1:47290";

/// A running dashboard and the process serving it.
pub struct Engine {
    pub address: url::Url,
    child: Child,
}

impl Engine {
    /// Start `mash dash` on the app's usual port, or on any free one.
    pub fn start() -> Result<Self, String> {
        let program = locate()?;
        Self::listen(&program, PREFERRED).or_else(|_| Self::listen(&program, "127.0.0.1:0"))
    }

    /// Start `mash dash` at a loopback address and wait for it to be ready.
    fn listen(program: &Path, address: &str) -> Result<Self, String> {
        let mut command = Command::new(program);
        command
            .args(["dash", "--listen", address])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // No console window for the engine.
            command.creation_flags(0x0800_0000);
        }
        let mut child = command
            .spawn()
            .map_err(|error| format!("Couldn’t start {}: {error}.", program.display()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or("The engine has no output to read.")?;
        let (found, address) = mpsc::channel();
        // `mash dash` writes one JSON line naming its address when its output is
        // not a terminal; everything it writes later is drained so it never blocks.
        std::thread::spawn(move || {
            let mut found = Some(found);
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Some(address) = found.as_ref().and_then(|_| dashboard_address(&line)) {
                    let _ = found.take().map(|found| found.send(address));
                } else if cfg!(debug_assertions) {
                    eprintln!("mash dash: {line}");
                }
            }
        });
        match address.recv_timeout(STARTUP) {
            Ok(address) => Ok(Self { address, child }),
            Err(_) => {
                let status = child.try_wait().ok().flatten();
                let _ = child.kill();
                let _ = child.wait();
                Err(match status {
                    Some(status) => {
                        format!("The Mash engine stopped before it was ready ({status}).")
                    }
                    None => "The Mash engine didn’t start within a minute.".to_owned(),
                })
            }
        }
    }

    /// Whether the engine has stopped on its own.
    pub fn stopped(&mut self) -> bool {
        !matches!(self.child.try_wait(), Ok(None))
    }

    /// Stop the engine with the app.
    pub fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The address in the startup line `mash dash` writes: `{"dashboard":"http://…/"}`.
/// Only a loopback http address is accepted.
pub fn dashboard_address(line: &str) -> Option<url::Url> {
    let value: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
    let address = url::Url::parse(value.get("dashboard")?.as_str()?).ok()?;
    let loopback = match address.host()? {
        url::Host::Ipv4(ip) => ip.is_loopback(),
        url::Host::Ipv6(ip) => ip.is_loopback(),
        url::Host::Domain(name) => name == "localhost",
    };
    (address.scheme() == "http" && loopback).then_some(address)
}

/// The `mash` binary that ships with the app, next to its own executable.
/// `MASH_DESKTOP_ENGINE` names another one while developing the app.
fn locate() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("MASH_DESKTOP_ENGINE") {
        return Ok(PathBuf::from(path));
    }
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let directory = executable.parent().unwrap_or(Path::new("."));
    let engine = directory.join(format!("mash{}", std::env::consts::EXE_SUFFIX));
    if engine.is_file() {
        Ok(engine)
    } else {
        Err(format!(
            "The Mash engine is missing from this app ({}).",
            engine.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::dashboard_address;

    #[test]
    fn reads_the_address_mash_dash_reports() {
        let address = dashboard_address(r#"{"dashboard":"http://127.0.0.1:53817/"}"#);
        assert_eq!(
            address.map(String::from).as_deref(),
            Some("http://127.0.0.1:53817/")
        );
        let warned = dashboard_address(r#"{"dashboard":"http://[::1]:8790/","warning":"x"}"#);
        assert!(warned.is_some());
    }

    #[test]
    fn ignores_other_lines_and_addresses_off_this_machine() {
        for line in [
            "",
            "Open the Mash dashboard in your browser.",
            r#"{"error":"busy"}"#,
            r#"{"dashboard":"http://192.168.1.20:8790/"}"#,
            r#"{"dashboard":"https://127.0.0.1:8790/"}"#,
            r#"{"dashboard":"file:///tmp/x"}"#,
        ] {
            assert!(dashboard_address(line).is_none(), "{line:?}");
        }
    }
}

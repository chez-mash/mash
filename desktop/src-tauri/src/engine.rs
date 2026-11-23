//! The Mash engine behind the window: the `mash` binary that ships inside the
//! app, running `mash dash` on a private loopback port. The window shows the
//! same dashboard a browser does, served by the same code; the app adds no
//! second implementation of anything the dashboard does.
//!
//! The app owns the engine it starts: it keeps the engine's last lines of
//! output for the error page, asks it to stop when the app quits, and records
//! its process number so an engine left behind by an app that did not quit
//! cleanly is stopped at the next start rather than holding the usual port.

use crate::stop;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex, PoisonError, mpsc};
use std::time::{Duration, Instant};

/// How long `mash dash` may take to say where it is listening.
const STARTUP: Duration = Duration::from_secs(60);
/// How long the engine has to finish its requests when asked to stop.
const STOPPING: Duration = Duration::from_secs(3);

/// The port the app asks for first. The dashboard keeps the reader's theme and
/// layout in browser storage, which belongs to one address, so a port that
/// stays the same keeps those choices from one start to the next. When the
/// port is taken the app uses any free one and the choices start fresh.
const PREFERRED: &str = "127.0.0.1:47290";

/// How many of the engine's last output lines are kept for the error page.
const LOG_LINES: usize = 40;
/// The longest output line kept, in characters.
const LOG_LINE: usize = 1000;

/// The engine's last lines of output, newest last.
pub type Log = Arc<Mutex<VecDeque<String>>>;

/// A running dashboard and the process serving it.
pub struct Engine {
    pub address: url::Url,
    child: Child,
    /// The file that records the engine's process number while it runs.
    pid_file: Option<PathBuf>,
}

impl Engine {
    /// Start `mash dash` on the app's usual port, or on any free one. The
    /// engine's output goes to `log`; `pid_file` records its process number.
    pub fn start(log: &Log, pid_file: Option<PathBuf>) -> Result<Self, String> {
        let program = locate()?;
        if let Some(file) = &pid_file {
            stop_leftover(file);
        }
        let mut engine = match Self::listen(&program, PREFERRED, log) {
            Ok(engine) => engine,
            Err(_) => {
                let engine = Self::listen(&program, "127.0.0.1:0", log)?;
                if port_taken(PREFERRED) {
                    remember(
                        log,
                        &format!(
                            "Port {} is in use by another program, so Mash is using {} \
                             instead; the dashboard's theme and layout start fresh there.",
                            PREFERRED, engine.address
                        ),
                    );
                }
                engine
            }
        };
        if let Some(file) = pid_file
            && std::fs::write(&file, engine.child.id().to_string()).is_ok()
        {
            engine.pid_file = Some(file);
        }
        Ok(engine)
    }

    /// Start `mash dash` at a loopback address and wait for it to be ready.
    fn listen(program: &Path, address: &str, log: &Log) -> Result<Self, String> {
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
        // not a terminal. Everything it writes is read so it never blocks, and
        // the last lines are kept for the error page.
        let output = log.clone();
        std::thread::spawn(move || {
            let mut found = Some(found);
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Some(address) = found.as_ref().and_then(|_| dashboard_address(&line)) {
                    let _ = found.take().map(|found| found.send(address));
                    continue;
                }
                if cfg!(debug_assertions) {
                    eprintln!("mash dash: {line}");
                }
                remember(&output, &line);
            }
        });
        match address.recv_timeout(STARTUP) {
            Ok(address) => Ok(Self {
                address,
                child,
                pid_file: None,
            }),
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

    /// How the engine ended, once it has stopped on its own.
    pub fn stopped(&mut self) -> Option<ExitStatus> {
        self.child.try_wait().ok().flatten()
    }

    /// Stop the engine with the app: ask it to finish (SIGTERM, or Ctrl+C on
    /// Windows), and end it only if it is still running after a few seconds.
    pub fn stop(&mut self) {
        if self.stopped().is_none() {
            stop::interrupt(self.child.id());
            let deadline = Instant::now() + STOPPING;
            while self.stopped().is_none() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(50));
            }
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        if let Some(file) = self.pid_file.take() {
            let _ = std::fs::remove_file(file);
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Keep one line of the engine's output, dropping the oldest past the limit.
pub fn remember(log: &Log, line: &str) {
    let mut lines = log.lock().unwrap_or_else(PoisonError::into_inner);
    lines.push_back(line.chars().take(LOG_LINE).collect());
    while lines.len() > LOG_LINES {
        lines.pop_front();
    }
}

/// Stop an engine that an earlier run of the app left behind, which would
/// otherwise keep the usual port and send this run to a new one.
///
/// The process number alone is not enough: the system may have given it to
/// another program since. The engine is stopped only when that process is a
/// `mash` and a Mash dashboard answers on the usual port; anything else on
/// the port is left alone and the app moves to a free one.
fn stop_leftover(pid_file: &Path) {
    let Some(pid) = std::fs::read_to_string(pid_file)
        .ok()
        .and_then(|text| text.trim().parse::<u32>().ok())
    else {
        return;
    };
    let _ = std::fs::remove_file(pid_file);
    if !stop::is_mash(pid) || !answers_as_mash(PREFERRED) {
        return;
    }
    stop::interrupt(pid);
    let deadline = Instant::now() + STOPPING;
    while stop::is_mash(pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    if stop::is_mash(pid) {
        stop::force(pid);
    }
    // Give the system a moment to release the port.
    let deadline = Instant::now() + Duration::from_secs(2);
    while answers_as_mash(PREFERRED) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Whether something is listening at `address`.
fn port_taken(address: &str) -> bool {
    address
        .parse::<SocketAddr>()
        .is_ok_and(|socket| TcpStream::connect_timeout(&socket, Duration::from_millis(500)).is_ok())
}

/// Whether a Mash dashboard answers its health check at `address`.
fn answers_as_mash(address: &str) -> bool {
    let Ok(socket) = address.parse::<SocketAddr>() else {
        return false;
    };
    let Ok(mut stream) = TcpStream::connect_timeout(&socket, Duration::from_millis(500)) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let request =
        format!("GET /api/health HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let mut reply = String::new();
    let _ = stream.take(64 * 1024).read_to_string(&mut reply);
    is_mash_health(&reply)
}

/// Whether an HTTP reply is the dashboard's `/api/health` answer:
/// `200 OK` with a JSON body naming the build.
fn is_mash_health(reply: &str) -> bool {
    let Some((head, body)) = reply.split_once("\r\n\r\n") else {
        return false;
    };
    let ok = head
        .lines()
        .next()
        .is_some_and(|status| status.split_whitespace().nth(1) == Some("200"));
    let build = serde_json::from_str::<serde_json::Value>(body.trim())
        .ok()
        .and_then(|value| value.get("build").map(serde_json::Value::is_string));
    ok && build == Some(true)
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
pub fn locate() -> Result<PathBuf, String> {
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
    use super::{Log, dashboard_address, is_mash_health, remember};

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

    #[test]
    fn keeps_only_the_last_lines_of_output() {
        let log = Log::default();
        for number in 0..100 {
            remember(&log, &format!("line {number}"));
        }
        remember(&log, &"x".repeat(5000));
        let lines = log.lock().unwrap();
        assert_eq!(lines.len(), 40);
        assert_eq!(lines.front().map(String::as_str), Some("line 61"));
        assert_eq!(lines.back().map(String::len), Some(1000));
    }

    #[test]
    fn knows_a_mash_dashboard_by_its_health_answer() {
        let reply = "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n{\"build\":\"abc\"}";
        assert!(is_mash_health(reply));
        for other in [
            "",
            "HTTP/1.1 404 Not Found\r\n\r\n{\"build\":\"abc\"}",
            "HTTP/1.1 200 OK\r\n\r\n<html>something else</html>",
            "HTTP/1.1 200 OK\r\n\r\n{\"status\":\"ok\"}",
        ] {
            assert!(!is_mash_health(other), "{other:?}");
        }
    }
}

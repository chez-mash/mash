use mash_model::AppError;
use serde::Serialize;
use std::io::{self, IsTerminal, Write};
use std::net::SocketAddr;

const ACCESS_WARNING: &str = "anyone who can reach this address can read your chat history; set --access-token-env to require sign-in";

#[derive(Serialize)]
struct Startup {
    dashboard: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<&'static str>,
}

pub(super) fn bind_error(address: SocketAddr, error: &io::Error) -> AppError {
    if error.kind() != io::ErrorKind::AddrInUse {
        return AppError::configuration(format!(
            "Could not listen for the dashboard at {address}. {error}"
        ));
    }

    let recovery = if address.ip().is_loopback() {
        let free_port = SocketAddr::new(address.ip(), 0);
        format!(
            "If Mash is already running, open its existing URL. To start a dashboard on a free local port, use `--listen \"{free_port}\"`."
        )
    } else {
        "Choose a free address or port with `--listen`.".to_owned()
    };
    AppError::configuration(format!(
        "The dashboard address {address} is already in use. {recovery}"
    ))
}

pub(super) fn report(address: SocketAddr, warn_about_access: bool) -> Result<(), AppError> {
    let mut stderr = io::stderr().lock();
    let startup = Startup {
        dashboard: format!("http://{address}/"),
        warning: warn_about_access.then_some(ACCESS_WARNING),
    };
    if stderr.is_terminal() {
        startup
            .write_terminal(&mut stderr, crate::has_web_dashboard())
            .map_err(AppError::io)
    } else {
        crate::write_json(&mut stderr, &startup)
    }
}

impl Startup {
    fn write_terminal(&self, writer: &mut impl Write, has_dashboard: bool) -> io::Result<()> {
        if let Some(warning) = self.warning {
            writeln!(writer, "{warning}")?;
        }
        if has_dashboard {
            writeln!(writer, "Open the Mash dashboard in your browser.")?;
            writeln!(writer, "{}", self.dashboard)?;
        } else {
            writeln!(writer, "This build does not include the web dashboard.")?;
            writeln!(writer, "The server is running at {}", self.dashboard)?;
            writeln!(
                writer,
                "Install a prebuilt Mash release to use the dashboard, or try `mash search`."
            )?;
        }
        writeln!(
            writer,
            "Keep this terminal open. Press Ctrl-C to stop the dashboard."
        )
    }
}

/// Ctrl+C, or SIGTERM where there is one: how a terminal, a service manager
/// and the desktop app ask the dashboard to stop.
pub(crate) async fn stop_requested() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut terminate) = signal(SignalKind::terminate()) {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = terminate.recv() => {}
            }
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_startup_reports_the_address_and_access_warning() {
        let dashboard = "http://127.0.0.1:40123/";
        for warning in [None, Some(ACCESS_WARNING)] {
            let startup = Startup {
                dashboard: dashboard.to_owned(),
                warning,
            };
            let mut report = Vec::new();
            startup.write_terminal(&mut report, true).unwrap();
            let report = String::from_utf8(report).unwrap();
            assert!(report.lines().any(|line| line == dashboard));
            assert!(report.contains("Press Ctrl-C to stop the dashboard."));
            assert_eq!(report.contains(ACCESS_WARNING), warning.is_some());

            let json = serde_json::to_value(&startup).unwrap();
            assert_eq!(json["dashboard"], dashboard);
            assert_eq!(
                json.get("warning").and_then(serde_json::Value::as_str),
                warning
            );
            assert_eq!(
                json.as_object().unwrap().len(),
                1 + usize::from(warning.is_some())
            );
        }
    }

    #[test]
    fn dashboard_startup_explains_the_missing_browser_bundle() {
        let startup = Startup {
            dashboard: "http://127.0.0.1:40123/".to_owned(),
            warning: None,
        };
        let mut report = Vec::new();
        startup.write_terminal(&mut report, false).unwrap();
        let report = String::from_utf8(report).unwrap();
        assert!(report.contains("This build does not include the web dashboard."));
        assert!(report.contains("prebuilt Mash release"));
        assert!(report.contains("`mash search`"));
    }
}

use super::{Dash, Paths, Reach, access, allowed_hosts, assistant, core, live, router, services};
use futures_util::{FutureExt, future::BoxFuture};
use mash_model::AppError;
use serde::Serialize;
use std::io::{self, IsTerminal, Write};
use std::net::SocketAddr;
use std::sync::Arc;

const ACCESS_WARNING: &str = "anyone who can reach this address can read your chat history; set --access-token-env to require sign-in";

#[derive(Serialize)]
struct Startup {
    dashboard: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<&'static str>,
}

/// Serves the dashboard until the process is told to stop.
///
/// The future is boxed here so that it is compiled in this crate, once. An
/// `async fn` is compiled where it is first polled, which for this one is the
/// executables' package: that package is rebuilt for every commit and would
/// compile the whole dashboard again each time.
pub fn run(
    paths: Paths,
    reach: Reach,
    services_config: Option<std::path::PathBuf>,
    fleet_config: Option<std::path::PathBuf>,
    fleet_node: Option<String>,
) -> BoxFuture<'static, Result<(), AppError>> {
    serve(paths, reach, services_config, fleet_config, fleet_node).boxed()
}

async fn serve(
    paths: Paths,
    reach: Reach,
    services_config: Option<std::path::PathBuf>,
    fleet_config: Option<std::path::PathBuf>,
    fleet_node: Option<String>,
) -> Result<(), AppError> {
    let Reach {
        listen,
        hosts: names,
        access_token_env,
        shared_only,
    } = reach;
    let access = access::Access::load(access_token_env.as_deref())?;
    if !names.is_empty() && !access.required() {
        return Err(AppError::configuration(
            "--allow-host makes the dashboard reachable from other devices; set --access-token-env as well",
        ));
    }
    let services = tokio::task::spawn_blocking(move || {
        if let Some(path) = fleet_config {
            services::Services::load_fleet(&path, fleet_node.as_deref())
        } else {
            services::Services::load(services_config.as_deref())
        }
    })
    .await
    .map_err(|_| AppError::internal("load services configuration"))??;
    let listener = tokio::net::TcpListener::bind(listen)
        .await
        .map_err(|error| bind_error(listen, &error))?;
    let address = listener
        .local_addr()
        .map_err(|_| AppError::internal("get dashboard address"))?;
    let loopback = address.ip().is_loopback() && names.is_empty();
    let mut hosts = allowed_hosts(address);
    hosts.extend(names);
    let warn_about_access = !loopback && !access.required();
    // Live updates watch the local index and, when one is configured, listen
    // to the shared one. A configuration that cannot be read leaves that out;
    // the history routes report what is wrong with it.
    let database =
        (!shared_only).then(|| paths.db.clone().unwrap_or_else(core::default_database_path));
    let config = paths.config.clone();
    let upstream = tokio::task::spawn_blocking(move || crate::config::load(config.as_deref()))
        .await
        .ok()
        .and_then(Result::ok)
        .and_then(|config| config.shared)
        .map(|config| live::Upstream {
            config,
            own: shared_only,
        });
    let state = Arc::new(Dash {
        console: crate::mcp::console::Console::new(paths.clone()),
        live: live::Live::new(database, upstream),
        questions: Arc::new(tokio::sync::Semaphore::new(assistant::MAX_QUESTIONS)),
        paths,
        hosts,
        loopback,
        access,
        shared_only,
        model_work: std::sync::Arc::new(tokio::sync::Semaphore::new(1)),
        services,
    });
    report(address, warn_about_access)?;
    let stopping = state.clone();
    axum::serve(listener, router(state))
        .with_graceful_shutdown(async move {
            stop_requested().await;
            // Open event streams would otherwise hold the server open.
            stopping.live.close();
        })
        .await
        .map_err(|_| AppError::internal("dashboard server failed"))
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

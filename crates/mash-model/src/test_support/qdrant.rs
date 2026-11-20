//! The Qdrant a backend test talks to: the one `MASH_TEST_QDRANT_URL` and
//! `MASH_TEST_QDRANT_GRPC_URL` name (CI's service, Windows' native Qdrant, or
//! one you run yourself), or else a disposable container started on first use.
//!
//! nextest runs each test in its own process, so the container belongs to one
//! test process. It is pinned (`qdrant/qdrant:v1.20.0`), capped at 2 GiB and
//! two CPUs, keeps Docker's disk-backed storage, and carries the
//! `mash.test=qdrant` and `mash.test.owner` labels CI's cleanup step matches.
//! A reaper process removes it when the test process exits, including after a
//! timeout or SIGKILL, which testcontainers' own cleanup on drop does not
//! cover: the container is never dropped here.
use std::sync::OnceLock;
use std::time::Duration;

const IMAGE: &str = "qdrant/qdrant";
const TAG: &str = "v1.20.0";
const MEMORY_BYTES: i64 = 2 * 1024 * 1024 * 1024;

/// Where a test's Qdrant answers.
pub struct Endpoints {
    pub url: String,
    grpc_url: Option<String>,
}

impl Endpoints {
    /// The gRPC endpoint. With `MASH_TEST_QDRANT_URL` set, it is
    /// `MASH_TEST_QDRANT_GRPC_URL`, which must then be set too.
    pub fn grpc_url(&self) -> &str {
        self.grpc_url.as_deref().expect(
            "MASH_TEST_QDRANT_URL is set without MASH_TEST_QDRANT_GRPC_URL; set both, or neither to start a container",
        )
    }
}

/// This test process's Qdrant, starting a container the first time it is
/// needed when no endpoint is configured.
pub fn endpoints() -> &'static Endpoints {
    static ENDPOINTS: OnceLock<Endpoints> = OnceLock::new();
    ENDPOINTS.get_or_init(|| {
        let configured = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
        match configured("MASH_TEST_QDRANT_URL") {
            Some(url) => Endpoints {
                url,
                grpc_url: configured("MASH_TEST_QDRANT_GRPC_URL"),
            },
            // The blocking runner keeps its own runtime, which must not start
            // inside a test's Tokio runtime.
            None => std::thread::spawn(start)
                .join()
                .expect("starting a disposable Qdrant panicked"),
        }
    })
}

#[cfg(unix)]
fn start() -> Endpoints {
    use testcontainers::core::{IntoContainerPort, WaitFor};
    use testcontainers::runners::SyncRunner;
    use testcontainers::{GenericImage, ImageExt};

    let owner = std::env::var("MASH_TEST_RESTART_OWNER")
        .unwrap_or_else(|_| format!("local-{}", std::process::id()));
    let container = GenericImage::new(IMAGE, TAG)
        .with_exposed_port(6333.tcp())
        .with_exposed_port(6334.tcp())
        // gRPC comes up last, so its line means both endpoints answer.
        .with_wait_for(WaitFor::message_on_stdout("Qdrant gRPC listening on 6334"))
        .with_label("mash.test", "qdrant")
        .with_label("mash.test.owner", owner)
        // Qdrant sizes its pools by the host's CPUs, not the container's.
        .with_env_var("QDRANT__SERVICE__MAX_WORKERS", "2")
        .with_host_config_modifier(|config| {
            config.memory = Some(MEMORY_BYTES);
            config.nano_cpus = Some(2_000_000_000);
        })
        // Generous: fsync stalls on a loaded build host delay startup.
        .with_startup_timeout(Duration::from_secs(120))
        .start()
        .unwrap_or_else(|error| {
            panic!(
                "could not start a disposable {IMAGE}:{TAG} with Docker ({error}). Start Docker and join the docker group, or set MASH_TEST_QDRANT_URL and MASH_TEST_QDRANT_GRPC_URL to a Qdrant you run"
            )
        });
    reap_on_exit(container.id());
    let host = container.get_host().expect("container host");
    let port = |internal: u16| {
        container
            .get_host_port_ipv4(internal.tcp())
            .expect("mapped Qdrant port")
    };
    let endpoints = Endpoints {
        url: format!("http://{host}:{}", port(6333)),
        grpc_url: Some(format!("http://{host}:{}", port(6334))),
    };
    // Dropping it would remove the container while tests still use it; the
    // reaper removes it once this process is gone.
    std::mem::forget(container);
    endpoints
}

#[cfg(not(unix))]
fn start() -> Endpoints {
    panic!(
        "set MASH_TEST_QDRANT_URL and MASH_TEST_QDRANT_GRPC_URL: disposable Qdrant containers need Docker with Linux containers"
    )
}

/// Remove the container once this process exits, however it exits. The
/// reaper leads its own process group, so nextest ending the test's group on
/// a timeout does not end it too.
#[cfg(unix)]
#[expect(
    clippy::zombie_processes,
    reason = "the reaper must outlive this process, so it is never waited for"
)]
fn reap_on_exit(container: &str) {
    use std::os::unix::process::CommandExt;
    let script = format!(
        "while kill -0 {} 2>/dev/null; do sleep 1; done; docker rm -f -v {container} >/dev/null 2>&1",
        std::process::id()
    );
    let mut command = if std::env::var("MASH_TEST_DOCKER_SG").as_deref() == Ok("1") {
        let mut command = std::process::Command::new("sg");
        command.args(["docker", "-c", &script]);
        command
    } else {
        let mut command = std::process::Command::new("sh");
        command.args(["-c", &script]);
        command
    };
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .process_group(0)
        .spawn()
        .expect("start the disposable Qdrant's reaper");
}

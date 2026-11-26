//! The MCP server on standard input and output, as an agent starts it.
use super::{AppError, Mash, OFFERED, Offered, Paths};
use futures_util::{FutureExt, future::BoxFuture};
use rmcp::ServiceExt;
use std::sync::Arc;

/// Serves the tools to the agent on standard input and output until it
/// disconnects.
///
/// The future is boxed here so that it is compiled in this crate, once. An
/// `async fn` is compiled where it is first polled, which for this one is the
/// executables' package: that package is rebuilt for every commit and would
/// compile the whole server again each time.
pub fn run(paths: Paths, offered: Offered) -> BoxFuture<'static, Result<(), AppError>> {
    serve(paths, offered).boxed()
}

async fn serve(paths: Paths, offered: Offered) -> Result<(), AppError> {
    // Set before the server exists; nothing has read it yet.
    let _ = OFFERED.set(offered);
    let registered_paths = paths.clone();
    let lease = crate::blocking(move || {
        crate::runtime::Lease::register(&registered_paths, crate::runtime::Kind::Mcp)
    })
    .await?;
    let server = Mash {
        paths: Arc::new(paths),
        gate: Arc::new(tokio::sync::Mutex::new(())),
        model_work: Arc::new(tokio::sync::Semaphore::new(1)),
        runtime: Some(Arc::new(lease)),
    };
    let service = server
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|_| AppError::internal("MCP initialization failed"))?;
    service
        .waiting()
        .await
        .map(|_| ())
        .map_err(|_| AppError::internal("MCP session failed"))
}

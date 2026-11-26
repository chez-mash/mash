//! The HTTP connections Mash keeps to Qdrant's REST API, and how long an
//! unused one may be sent on again.
//!
//! Qdrant closes a REST connection that has carried no request for five
//! seconds (measured on 1.20; it is not a Qdrant setting). A request sent on
//! such a connection while the close is still on its way is lost: it fails
//! with an incomplete response, which is not a timeout and so read as "could
//! not reach Qdrant" although Qdrant was running. A publication whose
//! preparation left the connection unused for about five seconds failed
//! this way. So Mash stops reusing a connection well before Qdrant closes it
//! and opens a new one instead, which costs one loopback handshake.
use crate::AppError;
use std::time::Duration;

/// How long an unused connection stays reusable. It leaves three seconds
/// between Mash's last use of a connection and Qdrant closing it.
const IDLE_CONNECTION: Duration = Duration::from_secs(2);

pub(super) fn client(headers: reqwest::header::HeaderMap) -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        // Internal database reads usually stay on the same host. Keep
        // their large snapshots free of compression CPU overhead.
        .no_gzip()
        .default_headers(headers)
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(IDLE_CONNECTION)
        .build()
        .map_err(|_| AppError::configuration("could not configure Qdrant HTTP client"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Read one request without a body; `false` when the peer closed first.
    async fn request(stream: &mut tokio::net::TcpStream) -> bool {
        let mut received = Vec::new();
        let mut buffer = [0; 1024];
        while !received.ends_with(b"\r\n\r\n") {
            match stream.read(&mut buffer).await {
                Ok(read) if read > 0 => received.extend_from_slice(&buffer[..read]),
                _ => return false,
            }
        }
        true
    }

    /// A server that answers the first request on each connection and closes
    /// the connection when a second one arrives on it, which is what a client
    /// sees when it sends as Qdrant's idle close is on its way. Returns its
    /// address and the number of connections it accepted.
    async fn closes_reused_connections() -> (String, Arc<AtomicUsize>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let accepted = Arc::new(AtomicUsize::new(0));
        let counter = accepted.clone();
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                counter.fetch_add(1, Ordering::SeqCst);
                tokio::spawn(async move {
                    if request(&mut stream).await {
                        let _ = stream
                            .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\n{}")
                            .await;
                        request(&mut stream).await;
                    }
                });
            }
        });
        (address, accepted)
    }

    #[tokio::test]
    async fn an_idle_connection_is_replaced_before_qdrant_closes_it() {
        let (address, accepted) = closes_reused_connections().await;
        let client = client(reqwest::header::HeaderMap::new()).unwrap();
        let get = || async {
            client
                .get(&address)
                .send()
                .await
                .map(|answer| answer.status())
        };
        assert_eq!(get().await.unwrap(), 200);
        // The pool measures idleness on the real clock.
        tokio::time::sleep(IDLE_CONNECTION + Duration::from_millis(250)).await;
        assert_eq!(get().await.unwrap(), 200);
        assert_eq!(accepted.load(Ordering::SeqCst), 2);
    }
}

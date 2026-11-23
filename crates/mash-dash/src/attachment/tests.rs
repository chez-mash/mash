use mash_app::attachments::{Availability, Metadata};

const PNG: &[u8] = &[137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13];
const NOTES: &[u8] = b"password=\"hunter2\"\nuseful notes";
/// A PDF as Claude Code keeps a `document` block: binary, so it is kept
/// byte for byte rather than as redacted text.
const PDF: &[u8] = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<< /Type /Catalog >>\nendobj\ntrailer\n<< /Root 1 0 R >>\n%%EOF\n";
const SVG: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script></svg>";

fn id(n: u8) -> String {
    format!("{n:02x}").repeat(32)
}

fn digest(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn kept(n: u8, name: &str, media_type: &str, bytes: &[u8]) -> Metadata {
    Metadata {
        id: id(n),
        name: name.into(),
        media_type: media_type.into(),
        availability: Availability::Retained,
        bytes: Some(bytes.len() as u64),
        digest: Some(digest(bytes)),
        redacted: false,
    }
}

/// One message with a retained image, text file, SVG and PDF, attachments
/// the configuration did not allow or found too large, and one whose
/// retained file has gone.
fn fixture() -> (tempfile::TempDir, crate::core::Paths) {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.toml");
    std::fs::write(&config, "version=1\nproviders={}\n").unwrap();
    let database = directory.path().join("index.sqlite3");
    drop(crate::storage::Storage::open(&database).unwrap());
    let attachments = vec![
        kept(1, "screenshot.png", "image/png", PNG),
        kept(2, "notes \u{e9}t\u{e9}.txt", "text/plain", NOTES),
        kept(3, "drawing.svg", "image/svg+xml", SVG),
        Metadata {
            id: id(4),
            name: "elsewhere.png".into(),
            media_type: "image/png".into(),
            availability: Availability::NotAllowed,
            bytes: None,
            digest: None,
            redacted: false,
        },
        kept(5, "gone.png", "image/png", b"no longer on disk"),
        kept(6, "Quarterly report.pdf", "application/pdf", PDF),
        Metadata {
            id: id(7),
            name: "scan.pdf".into(),
            media_type: "application/pdf".into(),
            availability: Availability::TooLarge,
            bytes: None,
            digest: None,
            redacted: false,
        },
    ];
    let root = mash_app::attachments::directory(&database);
    std::fs::create_dir_all(&root).unwrap();
    for bytes in [PNG, NOTES, SVG, PDF] {
        std::fs::write(root.join(digest(bytes)), bytes).unwrap();
    }
    let connection = rusqlite::Connection::open(&database).unwrap();
    crate::storage::packed::register(&connection).unwrap();
    connection
        .execute_batch("INSERT INTO chats(id,provider) VALUES('chat','claude-code');")
        .unwrap();
    connection
        .execute(
            "INSERT INTO messages(id,chat_id,ordinal,role,content_packed,attachments) VALUES('message','chat',0,'user','Look at these',?1)",
            [serde_json::to_string(&attachments).unwrap()],
        )
        .unwrap();
    drop(connection);
    let paths = crate::core::Paths {
        config: Some(config),
        db: Some(database),
        models_dir: None,
    };
    (directory, paths)
}

async fn serve(
    paths: crate::core::Paths,
    token: Option<&str>,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = crate::router(std::sync::Arc::new(crate::Dash {
        console: crate::mcp::console::Console::new(crate::core::Paths::default()),
        paths,
        hosts: crate::allowed_hosts(address),
        loopback: true,
        access: crate::access::Access::new(token),
        shared_only: false,
        model_work: std::sync::Arc::new(tokio::sync::Semaphore::new(1)),
        services: crate::services::Services::load(None).unwrap(),
        live: crate::live::Live::new(None, None),
        questions: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
    }));
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (address, server)
}

fn header<'r>(response: &'r reqwest::Response, name: &str) -> &'r str {
    response
        .headers()
        .get(name)
        .map_or("", |value| value.to_str().unwrap())
}

#[tokio::test]
async fn an_image_is_served_inline_and_kept_by_its_digest() {
    let (_directory, paths) = fixture();
    let (address, server) = serve(paths, None).await;
    let client = reqwest::Client::new();
    let url = format!(
        "http://{address}/api/attachment?message=message&attachment={}",
        id(1)
    );
    let response = client.get(&url).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(header(&response, "content-type"), "image/png");
    assert_eq!(header(&response, "content-disposition"), "inline");
    assert_eq!(header(&response, "x-content-type-options"), "nosniff");
    assert_eq!(
        header(&response, "content-security-policy"),
        "default-src 'none'; sandbox"
    );
    // Without the digest the page saw, the browser asks again each time.
    assert_eq!(header(&response, "cache-control"), "private, no-cache");
    let tag = format!("\"{}\"", digest(PNG));
    assert_eq!(header(&response, "etag"), tag);
    assert_eq!(response.bytes().await.unwrap().as_ref(), PNG);

    let unchanged = client
        .get(&url)
        .header("if-none-match", &tag)
        .send()
        .await
        .unwrap();
    assert_eq!(unchanged.status(), reqwest::StatusCode::NOT_MODIFIED);
    assert!(unchanged.bytes().await.unwrap().is_empty());

    // Asked for as a download, as the Download button does, the same picture
    // is an attachment named for the file, which every webview saves.
    let saved = client
        .get(format!("{url}&download=true"))
        .send()
        .await
        .unwrap();
    assert_eq!(header(&saved, "content-type"), "image/png");
    assert!(header(&saved, "content-disposition").starts_with("attachment; filename="));

    let pinned = client
        .get(format!("{url}&digest={}", digest(PNG)))
        .send()
        .await
        .unwrap();
    assert_eq!(pinned.status(), reqwest::StatusCode::OK);
    assert_eq!(
        header(&pinned, "cache-control"),
        "private, max-age=31536000, immutable"
    );
    // A digest the attachment no longer has is not answered with other bytes.
    let changed = client
        .get(format!("{url}&digest={}", "0".repeat(64)))
        .send()
        .await
        .unwrap();
    assert_eq!(changed.status(), reqwest::StatusCode::NOT_FOUND);
    server.abort();
}

#[tokio::test]
async fn other_files_are_downloads_with_their_text_redacted() {
    let (_directory, paths) = fixture();
    let (address, server) = serve(paths, None).await;
    let client = reqwest::Client::new();
    let get = |n: u8| {
        client
            .get(format!(
                "http://{address}/api/attachment?message=message&attachment={}",
                id(n)
            ))
            .send()
    };
    let pdf = get(6).await.unwrap();
    assert_eq!(pdf.status(), reqwest::StatusCode::OK);
    assert_eq!(header(&pdf, "content-type"), "application/pdf");
    assert_eq!(
        header(&pdf, "content-disposition"),
        "attachment; filename=\"Quarterly report.pdf\"; filename*=UTF-8''Quarterly%20report.pdf"
    );
    assert_eq!(header(&pdf, "x-content-type-options"), "nosniff");
    assert_eq!(pdf.bytes().await.unwrap().as_ref(), PDF);

    let notes = get(2).await.unwrap();
    assert_eq!(notes.status(), reqwest::StatusCode::OK);
    assert_eq!(header(&notes, "content-type"), "text/plain");
    assert_eq!(
        header(&notes, "content-disposition"),
        "attachment; filename=\"notes _t_.txt\"; filename*=UTF-8''notes%20%C3%A9t%C3%A9.txt"
    );
    assert_eq!(header(&notes, "x-content-type-options"), "nosniff");
    let text = notes.text().await.unwrap();
    assert!(
        text.contains("useful notes") && !text.contains("hunter2"),
        "{text}"
    );

    // An SVG can carry script, so it is never shown inline.
    let svg = get(3).await.unwrap();
    assert_eq!(svg.status(), reqwest::StatusCode::OK);
    assert_eq!(header(&svg, "content-type"), "image/svg+xml");
    assert!(header(&svg, "content-disposition").starts_with("attachment; "));
    assert_eq!(
        header(&svg, "content-security-policy"),
        "default-src 'none'; sandbox"
    );
    server.abort();
}

#[tokio::test]
async fn attachments_without_bytes_are_refused_as_errors() {
    let (_directory, paths) = fixture();
    let (address, server) = serve(paths, None).await;
    let client = reqwest::Client::new();
    for (message, attachment) in [
        ("message", id(4)),
        ("message", id(5)),
        ("message", id(7)),
        ("message", id(9)),
        ("missing", id(1)),
    ] {
        let response = client
            .get(format!(
                "http://{address}/api/attachment?message={message}&attachment={attachment}"
            ))
            .send()
            .await
            .unwrap();
        assert!(
            response.status().is_client_error(),
            "{message}/{attachment}: {}",
            response.status()
        );
        assert_eq!(header(&response, "content-type"), "application/json");
        assert_eq!(header(&response, "cache-control"), "no-store");
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body["error"]["message"].is_string(), "{body}");
    }
    server.abort();
}

#[tokio::test]
async fn attachments_need_the_same_sign_in_as_the_chat() {
    let (_directory, paths) = fixture();
    let (address, server) = serve(paths, Some("0123456789abcdef0123456789abcdef")).await;
    let response = reqwest::Client::new()
        .get(format!(
            "http://{address}/api/attachment?message=message&attachment={}",
            id(1)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    server.abort();
}

#[test]
fn a_peer_answer_keeps_only_headers_this_dashboard_would_send() {
    let mut from = reqwest::header::HeaderMap::new();
    from.insert("content-type", "text/html".parse().unwrap());
    from.insert("content-disposition", "inline".parse().unwrap());
    from.insert("cache-control", "public, max-age=99".parse().unwrap());
    from.insert("etag", "\"not a digest\"".parse().unwrap());
    let page = super::relay(&from, b"<script>".to_vec());
    assert_eq!(page.headers()["content-type"], "text/html");
    assert_eq!(page.headers()["content-disposition"], "attachment");
    assert_eq!(page.headers()["cache-control"], "private, no-cache");
    assert!(page.headers().get("etag").is_none());

    let tag = format!("\"{}\"", digest(PNG));
    from.insert("content-type", "image/png".parse().unwrap());
    from.insert("cache-control", super::KEPT.parse().unwrap());
    from.insert("etag", tag.parse().unwrap());
    let image = super::relay(&from, PNG.to_vec());
    assert_eq!(image.headers()["content-disposition"], "inline");
    assert_eq!(image.headers()["cache-control"], super::KEPT);
    assert_eq!(image.headers()["etag"], tag.as_str());

    // A peer that served the picture as a download keeps it one.
    from.insert(
        "content-disposition",
        "attachment; filename=\"shot.png\"".parse().unwrap(),
    );
    let saved = super::relay(&from, PNG.to_vec());
    assert_eq!(saved.headers()["content-disposition"], "attachment");
}

use super::super::{Client, Request, http_client};
use super::*;
use crate::observability::retrieval::RerankerFallback;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

const TOKEN: &str = "worker-token";

/// Stands in for the model: each document scores its own length.
fn lengths(calls: &Arc<AtomicUsize>) -> Scorer {
    let calls = Arc::clone(calls);
    Arc::new(move |_, documents| {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(documents
            .iter()
            .map(|text| f32::from(u16::try_from(text.len()).unwrap()))
            .collect())
    })
}

async fn start(scorer: Scorer) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router(TOKEN, scorer)).await });
    format!("http://{address}/rerank")
}

fn client(url: &str, token: &str) -> Client {
    Client {
        http: http_client().unwrap(),
        url: url.to_owned(),
        token: token.to_owned(),
        timeout: Duration::from_secs(5),
    }
}

#[test]
fn a_client_frame_decodes_to_its_query_and_documents_in_order() {
    let documents = ["first", "東京", ""];
    let request = Request::new("寿司 recipe", &documents);
    let batch = decode(&request.encode()).unwrap();
    assert_eq!(&batch.request_id, request.id.as_bytes());
    assert_eq!(batch.query, "寿司 recipe");
    assert_eq!(batch.documents, documents);
    let reply = encode_reply(&batch.request_id, 7, &[1.5, -2.0, 0.0]).unwrap();
    let scores = request.validate(&reply).unwrap();
    assert_eq!(scores.scores, [1.5, -2.0, 0.0]);
    assert_eq!(scores.inference_milliseconds, 7);
}

#[test]
fn malformed_and_foreign_frames_are_refused() {
    let valid = Request::new("q", &["x", "y"]).encode();
    assert!(decode(&valid).is_ok());
    for malformed in [
        &valid[..0],
        &valid[..5],
        &valid[..valid.len() - 1],
        &[valid.as_slice(), b"x"].concat(),
        &[b"MSH1", &valid[4..]].concat(),
    ] {
        assert_eq!(decode(malformed).unwrap_err(), Refusal::Malformed);
    }
    let mut foreign = valid.clone();
    foreign[6] ^= 1;
    assert_eq!(decode(&foreign).unwrap_err(), Refusal::Recipe);
    // The second document's ID sits before its four-byte length and one-byte text.
    let mut duplicate = valid.clone();
    let id = valid.len() - 9;
    duplicate[id..id + 4].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(decode(&duplicate).unwrap_err(), Refusal::Malformed);
    let mut out_of_range = valid;
    out_of_range[id..id + 4].copy_from_slice(&2u32.to_le_bytes());
    assert_eq!(decode(&out_of_range).unwrap_err(), Refusal::Malformed);
    // A query needs documents, and documents need a query.
    let mut empty_query = Request::new("", &["x"]).encode();
    assert_eq!(decode(&empty_query).unwrap_err(), Refusal::Malformed);
    empty_query.truncate(empty_query.len() - 13);
    empty_query.extend_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        decode(&empty_query).unwrap().documents,
        Vec::<String>::new()
    );
}

#[test]
fn byte_limits_are_refused_by_name() {
    let long = "é".repeat(MAX_QUERY_BYTES / 2 + 1);
    assert_eq!(
        decode(&Request::new(&long, &["x"]).encode()).unwrap_err(),
        Refusal::QueryLimit
    );
    let large = "é".repeat(MAX_DOCUMENT_BYTES / 2 + 1);
    assert_eq!(
        decode(&Request::new("q", &[large.as_str()]).encode()).unwrap_err(),
        Refusal::DocumentLimit
    );
}

#[test]
fn a_reply_never_carries_a_score_the_client_would_refuse() {
    let id = [7u8; REQUEST_ID_BYTES];
    for invalid in [f32::NAN, f32::INFINITY, 1001.0] {
        assert!(encode_reply(&id, 0, &[0.0, invalid]).is_none());
    }
    assert_eq!(encode_reply(&id, 0, &[]).unwrap().len(), 4 + 32 + 8);
}

#[tokio::test]
async fn the_remote_client_gets_scores_from_the_worker_in_input_order() {
    let calls = Arc::new(AtomicUsize::new(0));
    let url = start(lengths(&calls)).await;
    let scores = client(&url, TOKEN)
        .rerank("q", &["four", "sixsix", "x"])
        .await
        .unwrap();
    assert_eq!(scores.scores, [4.0, 6.0, 1.0]);
    // Large enough to be sent compressed.
    let large = "word ".repeat(190);
    let scores = client(&url, TOKEN)
        .rerank("q", &[large.as_str(), "x"])
        .await
        .unwrap();
    assert_eq!(scores.scores, [950.0, 1.0]);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn a_wrong_or_missing_token_never_reaches_the_model() {
    let calls = Arc::new(AtomicUsize::new(0));
    let url = start(lengths(&calls)).await;
    for token in ["wrong", "", "worker-token-and-more", "worker"] {
        assert_eq!(
            client(&url, token).rerank("q", &["x"]).await.unwrap_err(),
            RerankerFallback::Unavailable
        );
    }
    let bare = reqwest::Client::new()
        .post(&url)
        .body(Request::new("q", &["x"]).encode())
        .send()
        .await
        .unwrap();
    assert_eq!(bare.status(), 401);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn warm_ups_and_refused_requests_run_no_inference() {
    let calls = Arc::new(AtomicUsize::new(0));
    let url = start(lengths(&calls)).await;
    let http = reqwest::Client::new();
    let post = |body: Vec<u8>, encoding: Option<&'static str>| {
        let mut request = http.post(&url).bearer_auth(TOKEN).body(body);
        if let Some(encoding) = encoding {
            request = request.header("content-encoding", encoding);
        }
        request.send()
    };
    let mut warm_up = Request::new("", &[]).encode();
    let response = post(warm_up.clone(), None).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.bytes().await.unwrap().len(), 4 + 32 + 8);
    warm_up[0] = b'X';
    assert_eq!(post(warm_up, None).await.unwrap().status(), 400);
    let valid = Request::new("q", &["x"]).encode();
    assert_eq!(
        post(valid.clone(), Some("gzip")).await.unwrap().status(),
        415
    );
    assert_eq!(post(valid, Some("zstd")).await.unwrap().status(), 400);
    assert_eq!(
        post(vec![0; MAX_BODY_BYTES + 1], None)
            .await
            .unwrap()
            .status(),
        413
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_failing_or_out_of_range_model_is_an_error_not_a_score() {
    let failing: Scorer = Arc::new(|_, _| Err(AppError::model("broken")));
    let url = start(failing).await;
    assert_eq!(
        client(&url, TOKEN).rerank("q", &["x"]).await.unwrap_err(),
        RerankerFallback::Unavailable
    );
    let wild: Scorer = Arc::new(|_, documents| Ok(vec![f32::NAN; documents.len()]));
    let url = start(wild).await;
    assert_eq!(
        client(&url, TOKEN).rerank("q", &["x"]).await.unwrap_err(),
        RerankerFallback::Unavailable
    );
}

#[tokio::test]
async fn the_worker_refuses_to_listen_beyond_this_machine() {
    let error = serve("0.0.0.0:8000".parse().unwrap(), PathBuf::new())
        .await
        .unwrap_err();
    assert!(error.error.message.contains("this machine only"));
}

/// `deployment/reranking/openapi.json` is the committed copy of the worker
/// contract. Rewrite it with `MASH_WRITE_CONTRACT=1` after changing a limit,
/// a frame or a status.
#[test]
fn committed_worker_contract_matches_the_code() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../deployment/reranking/openapi.json");
    let mut current = serde_json::to_string_pretty(&document()).unwrap();
    current.push('\n');
    if std::env::var_os("MASH_WRITE_CONTRACT").is_some() {
        std::fs::write(&path, &current).unwrap();
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        committed == current,
        "deployment/reranking/openapi.json is out of date: rerun this test with MASH_WRITE_CONTRACT=1"
    );
}

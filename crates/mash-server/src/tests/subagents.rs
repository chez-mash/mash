//! Other machines' subagent chats, as the shared index marks them: in search
//! hits, in `list_chats` and in the dashboard catalogue.
use super::*;

/// The fixture snapshot with a subagent `child` of its `conversation`.
fn marked_snapshot(revision: i64) -> Vec<Value> {
    let mut events = snapshot(revision);
    events[1]["records"].as_array_mut().unwrap().push(json!({
        "id":"child","provider":"codex","title":"Child task","project":"/work/mash",
        "repository":"repo://mash","created_at":1,"updated_at":3,
        "subagent":{"parent_chat_id":"conversation","label":"Tidy the docs","role":"worker","depth":1}
    }));
    let mut vector = vec![0.0; 384];
    vector[0] = 127.0;
    events[2]["records"].as_array_mut().unwrap().push(json!({
        "id":"child-message","chat_id":"child","ordinal":0,"role":"user",
        "content":"hello child task","projection":"hello child task","created_at":3,
        "generation":crate::semantic::SHARED_EMBEDDING_GENERATION,"vector":vector
    }));
    events
}

fn rows<'a>(rows: &'a Value, id: &str) -> Vec<&'a Value> {
    rows.as_array()
        .unwrap()
        .iter()
        .filter(|row| row["chat_id"] == id)
        .collect()
}

async fn chats(service: &Service, request: &Value, markers: bool) -> Value {
    let mut builder = service.request("chats", SECOND).json(request);
    if markers {
        builder = builder.header(crate::shared::protocol::SUBAGENT_MARKERS_HEADER, "1");
    }
    let page: Value = builder
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    page["chats"].clone()
}

#[tokio::test]
async fn qdrant_shared_chat_lists_mark_subagents_for_clients_that_read_markers() {
    let service = Service::qdrant(384).await;
    assert_eq!(
        service.upload(FIRST, &marked_snapshot(1)).await.status(),
        StatusCode::OK
    );
    let listed = chats(&service, &json!({}), true).await;
    let child = rows(&listed, "child");
    assert_eq!(child[0]["subagent"]["label"], "Tidy the docs");
    assert_eq!(child[0]["subagent"]["parent_title"], "Shared history");
    assert_eq!(rows(&listed, "conversation")[0]["subagents"], 1);

    let people = chats(&service, &json!({"exclude_subagents":true}), true).await;
    assert_eq!(rows(&people, "child").len(), 0);
    assert_eq!(rows(&people, "conversation").len(), 1);

    // An older client rejects fields it does not know in a row: it gets none,
    // and its exclusion still holds.
    let older = chats(&service, &json!({}), false).await;
    let child = rows(&older, "child");
    assert!(child[0].get("subagent").is_none());
    assert!(rows(&older, "conversation")[0].get("subagents").is_none());
    let older_people = chats(&service, &json!({"exclude_subagents":true}), false).await;
    assert_eq!(rows(&older_people, "child").len(), 0);

    let catalog: Value = reqwest::Client::new()
        .get(format!("{}/v1/catalog", service.url))
        .bearer_auth(SECOND)
        .header(crate::shared::protocol::SUBAGENT_MARKERS_HEADER, "1")
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(rows(&catalog, "child")[0]["subagent"]["role"], "worker");
    let older = service.get("catalog", SECOND).await;
    assert!(rows(&older, "child")[0].get("subagent").is_none());

    // The next publication without the marker leaves the chat unmarked.
    let mut unmarked = marked_snapshot(2);
    unmarked[1]["records"][1]
        .as_object_mut()
        .unwrap()
        .remove("subagent");
    assert_eq!(
        service.upload(FIRST, &unmarked).await.status(),
        StatusCode::OK
    );
    let listed = chats(&service, &json!({"exclude_subagents":true}), true).await;
    assert!(rows(&listed, "child")[0].get("subagent").is_none());
    service.cleanup().await;
}

#[tokio::test]
async fn qdrant_shared_search_hits_carry_the_marker_their_machine_published() {
    let service = Service::qdrant(384).await;
    assert_eq!(
        service.upload(FIRST, &marked_snapshot(1)).await.status(),
        StatusCode::OK
    );
    let mut vector = vec![0.0; 384];
    vector[0] = 127.0;
    let found: Value = service
        .request("search", SECOND)
        .json(&json!({"query":"hello", "vector":vector}))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    for kind in ["lexical", "semantic"] {
        let hits = found[kind].as_array().unwrap();
        let child: Vec<_> = hits
            .iter()
            .filter(|hit| hit["chat_id"] == "child")
            .collect();
        assert!(!child.is_empty(), "{kind}: {found}");
        for hit in child {
            assert_eq!(
                hit["metadata"]["subagent"]["label"], "Tidy the docs",
                "{kind}"
            );
            assert_eq!(
                hit["metadata"]["subagent"]["parent_chat_id"], "conversation",
                "{kind}"
            );
        }
        for hit in hits.iter().filter(|hit| hit["chat_id"] == "conversation") {
            assert!(hit["metadata"].get("subagent").is_none(), "{kind}");
        }
    }
    service.cleanup().await;
}

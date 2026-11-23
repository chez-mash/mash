//! The server's route table, and the contract derived from it.
use super::chats::{catalog, list_chats};
use super::{
    App, admin, attachments, chat_page, contexts, events, fingerprints, freshness, get_record,
    group_status, join, list_records, query, search, sync, view,
};
use axum::Router;

/// Every route, and the `OpenAPI` document `aide` derives from the handlers'
/// own request and answer types. The synchronization route upgrades to a
/// WebSocket; its frames are outside what the document describes.
pub(crate) fn api() -> (Router<App>, aide::openapi::OpenApi) {
    use aide::axum::routing::{get, post};
    let mut document = aide::openapi::OpenApi {
        info: aide::openapi::Info {
            title: "mash-server".into(),
            description: Some(
                "The shared index's API. Every request carries a node's bearer token. Generated from the server's routes; do not edit."
                    .into(),
            ),
            version: "1".into(),
            ..Default::default()
        },
        ..Default::default()
    };
    let router = aide::axum::ApiRouter::new()
        .api_route("/v1/freshness", get(freshness))
        .api_route("/v1/search", post(search))
        .api_route("/v1/chats", post(list_chats))
        .api_route("/v1/records", post(list_records))
        .api_route("/v1/record", post(get_record))
        .api_route("/v1/catalog", get(catalog))
        .api_route("/v1/chat", post(chat_page))
        .api_route("/v1/view", post(view))
        .api_route("/v1/contexts", post(contexts::read))
        .api_route("/v1/query", post(query))
        .api_route("/v1/fingerprints", get(fingerprints))
        .api_route("/v1/attachments", post(attachments::list))
        .api_route("/v1/attachment", post(attachments::get))
        .api_route("/v1/sync", get(sync::upgrade))
        .api_route("/v1/events", get(events::listen))
        .api_route("/v1/join/check", post(join::check))
        .api_route("/v1/join", post(join::redeem))
        .api_route("/v1/status", get(group_status::read))
        .api_route("/v1/admin/machines", get(admin::overview))
        .api_route("/v1/admin/add", post(admin::add))
        .api_route("/v1/admin/replace", post(admin::replace))
        .api_route("/v1/admin/remove", post(admin::remove))
        .api_route("/v1/admin/delete", post(admin::delete))
        .finish_api(&mut document);
    (router, document)
}

/// The contract as JSON, with each schema under a name of its own.
#[cfg(test)]
pub(crate) fn document() -> serde_json::Value {
    let mut document = serde_json::to_value(api().1).unwrap_or_default();
    crate::openapi::settle(
        &mut document,
        "Request",
        &[
            ("direction", "ChatListRequest"),
            ("group_by", "QueryRequest"),
            ("attachment_id", "AttachmentRequest"),
        ],
    );
    crate::openapi::settle(&mut document, "Filter", &[("chat", "QueryFilter")]);
    crate::openapi::settle(&mut document, "Scope", &[("after", "CoverageScope")]);
    crate::openapi::sorted(document)
}

use crate::state::AppState;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use serde_json::json;

pub async fn well_known(State(state): axum::extract::State<AppState>) -> impl IntoResponse {
    let nodeinfo = json!({
        "links": [
            {
                "rel": "http://nodeinfo.diaspora.software/ns/schema/2.1",
                "href": format!("https://{}/nodeinfo/2.1", state.domain)
            }
        ]
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/activity+json"),
    );

    (headers, Json(nodeinfo)).into_response()
}

pub async fn nodeinfo(State(state): axum::extract::State<AppState>) -> impl IntoResponse {
    let nodeinfo = json!({
        "version": "2.1",
        "software": {
            "name": "tinyap",
            "version": "0.1.0"
        },
        "protocols": [
            "activitypub"
        ],
        "services": {
            "inbound": [],
            "outbound": []
        },
        "openRegistrations": state.config.allow_signup,
        "usage": {
            "users": {
                "total": sqlx::query!("SELECT COUNT(*) as count FROM users WHERE is_local = 1")
                    .fetch_one(&state.db_pool)
                    .await
                    .expect("Failed to fetch user count")
                    .count
            },
        },
        "metadata": {}
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/activity+json"),
    );

    (headers, Json(nodeinfo)).into_response()
}

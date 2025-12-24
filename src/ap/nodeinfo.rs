use crate::state::AppState;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use serde_json::json;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

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
    let total_users = sqlx::query!("SELECT COUNT(*) as count FROM users WHERE is_local = 1")
        .fetch_one(&state.db_pool)
        .await
        .expect("Failed to fetch user count")
        .count;

    let month_ago = OffsetDateTime::now_utc() - time::Duration::days(30);
    let month_ago = month_ago.format(&Rfc3339).unwrap();
    let active_users = sqlx::query!(
        "SELECT COUNT(DISTINCT users.id) as count
        FROM users
        JOIN notes ON notes.user_id = users.id
        AND users.is_local=1
        WHERE notes.created_at > ?;",
        month_ago
    )
    .fetch_one(&state.db_pool)
    .await
    .expect("Failed to fetch active user count")
    .count;

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
                "total": total_users,
                "activeMonth": active_users
            },
        },
        "metadata": {
            "nodeName": state.site_name,
        }
    });

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/activity+json"),
    );

    (headers, Json(nodeinfo)).into_response()
}

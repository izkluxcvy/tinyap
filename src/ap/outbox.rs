use crate::state::AppState;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

#[derive(serde::Deserialize)]
pub struct PageParam {
    page: Option<bool>,
}

pub async fn api(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(PageParam { page }): Query<PageParam>,
) -> impl IntoResponse {
    let user = sqlx::query!(
        "SELECT id, actor_id FROM users WHERE username = ?",
        username
    )
    .fetch_optional(&state.db_pool)
    .await
    .expect("Failed to fetch user from database");

    let Some(user) = user else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "User not found"})),
        )
            .into_response();
    };

    if let Some(true) = page {
        let notes = sqlx::query!(
            "SELECT uuid, content, created_at
            FROM notes
            WHERE user_id = ?
            ORDER BY created_at DESC",
            user.id
        )
        .fetch_all(&state.db_pool)
        .await
        .expect("Failed to fetch notes from database");

        let create_activities: Vec<_> = notes
            .iter()
            .map(|note| {
                json!({
                    "id": format!("https://{}/notes/{}", state.domain, note.uuid),
                    "type": "Create",
                    "actor": user.actor_id,
                    "published": note.created_at,
                    "to": ["https://www.w3.org/ns/activitystreams#Public"],
                    "object": {
                        "id": format!("https://{}/notes/{}", state.domain, note.uuid),
                        "type": "Note",
                        "attributedTo": user.actor_id,
                        "content": note.content,
                        "published": note.created_at,
                        "to": ["https://www.w3.org/ns/activitystreams#Public"],
                    }
                })
            })
            .collect();

        let outbox_id = format!("{}/outbox", user.actor_id);
        let outbox_json = json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": format!("{}?page=true", outbox_id),
            "type": "OrderedCollectionPage",
            "partOf": outbox_id,
            "orderedItems": create_activities,
            "next": null,
            "prev": null
        });

        let mut json_headers = HeaderMap::new();
        json_headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/activity+json"),
        );
        (json_headers, Json(outbox_json)).into_response()
    } else {
        let outbox_id = format!("{}/outbox", user.actor_id);
        let outbox_json = json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": outbox_id,
            "type": "OrderedCollection",
            "totalItems": 1,
            "first": format!("{}?page=true", outbox_id),
        });

        let mut json_headers = HeaderMap::new();
        json_headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/activity+json"),
        );
        (json_headers, Json(outbox_json)).into_response()
    }
}

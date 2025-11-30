use crate::state::AppState;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

pub async fn api(State(state): State<AppState>, Path(uuid): Path<String>) -> impl IntoResponse {
    let note = sqlx::query!(
        "SELECT notes.content, notes.created_at, users.actor_id
        FROM notes
        JOIN users ON users.id = notes.user_id
        WHERE notes.uuid = ?",
        uuid
    )
    .fetch_optional(&state.db_pool)
    .await
    .expect("Failed to fetch note from database");

    let Some(note) = note else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Note not found"})),
        )
            .into_response();
    };

    let json_body = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("https://{}/notes/{}", state.domain, uuid),
        "type": "Note",
        "url": format!("https://{}/notes/{}", state.domain, uuid),
        "attributedTo": note.actor_id,
        "content": note.content,
        "published": note.created_at,
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
    });

    let mut json_headers = HeaderMap::new();
    json_headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/activity+json"),
    );
    (json_headers, Json(json_body)).into_response()
}

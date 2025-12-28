use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

pub async fn get(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    // Get note
    let note = queries::note::get_by_id(&state, id).await;

    let Some(note) = note else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "note not found"})),
        )
            .into_response();
    };

    // Check if public
    if note.is_public == 0 {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "note is private"})),
        )
            .into_response();
    }

    // Get author
    let author = queries::user::get_by_id(&state, note.author_id).await;

    // Response
    let mut json_headers = HeaderMap::new();
    json_headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/activity+json"),
    );
    let json_body = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": &note.ap_url,
        "type": "Note",
        "url": &utils::note_url(&state.domain, &author.username, note.id),
        "attributedTo": &author.ap_url,
        "content": &note.content,
        "published": &note.created_at,
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
    });

    (json_headers, Json(json_body)).into_response()
}

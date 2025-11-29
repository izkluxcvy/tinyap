use crate::state::AppState;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

pub async fn api(State(state): State<AppState>, Path(username): Path<String>) -> impl IntoResponse {
    let row = sqlx::query!(
        "SELECT id, display_name, bio, actor_id, public_key
        FROM users
        WHERE username = ?",
        username
    )
    .fetch_optional(&state.db_pool)
    .await
    .expect("Failed to fetch user from database");

    let Some(user) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "User not found"})),
        )
            .into_response();
    };

    let actor_id = user.actor_id;
    let mut json_headers = HeaderMap::new();
    json_headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/activity+json"),
    );
    let json_body = json!({
        "@context": [
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1"
        ],
        "id": actor_id,
        "type": "Person",
        "preferredUsername": username,
        "name": user.display_name,
        "summary": user.bio,
        "inbox": format!("{}/inbox", actor_id),
        "outbox": format!("{}/outbox", actor_id),
        "followers": format!("{}/followers", actor_id),
        "following": format!("{}/following", actor_id),
        "publicKey": {
            "id": format!("{}#main-key", actor_id),
            "owner": actor_id,
            "publicKeyPem": user.public_key,
        },
    });

    (json_headers, Json(json_body)).into_response()
}

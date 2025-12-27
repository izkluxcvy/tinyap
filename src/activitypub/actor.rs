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

pub async fn get(State(state): State<AppState>, Path(username): Path<String>) -> impl IntoResponse {
    // Get user
    let user = queries::user::get_by_username(&state, &username).await;

    let Some(user) = user else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "user not found"})),
        )
            .into_response();
    };

    // Response
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
        "id": &user.ap_url,
        "type": "Person",
        "url": &utils::user_url(&state.domain, &user.username),
        "preferredUsername": &user.username,
        "name": &user.display_name,
        "summary": &user.bio,
        "inbox": &user.inbox_url,
        "outbox": &utils::local_user_outbox_url(&state.domain, &user.username),
        "publicKey": {
            "id": &format!("{}#main-key", &user.ap_url),
            "owner": &user.ap_url,
            "publicKeyPem": &user.public_key,
        },
    });

    (json_headers, Json(json_body)).into_response()
}

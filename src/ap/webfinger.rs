use crate::state::AppState;
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

#[derive(serde::Deserialize)]
pub struct WebFingerParam {
    resource: String,
}

pub async fn api(
    State(state): State<AppState>,
    Query(WebFingerParam { resource }): Query<WebFingerParam>,
) -> impl IntoResponse {
    if !resource.starts_with("acct:") {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid resource format"})),
        )
            .into_response();
    }

    let acct = resource.trim_start_matches("acct:");
    let parts: Vec<&str> = acct.split('@').collect();

    if parts.len() != 2 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid acct format"})),
        )
            .into_response();
    }

    let username = parts[0].to_string();
    let user = sqlx::query!("SELECT actor_id FROM users WHERE username = ?", username)
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

    let mut json_headers = HeaderMap::new();
    json_headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/jrd+json"),
    );
    let json_body = json!({
        "subject": resource,
        "links": [
            {
                "rel": "self",
                "type": "application/activity+json",
                "href": user.actor_id
            }
        ]
    });

    (json_headers, Json(json_body)).into_response()
}

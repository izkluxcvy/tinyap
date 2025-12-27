use crate::back::init::AppState;
use crate::back::queries;

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

#[derive(serde::Deserialize)]
pub struct WebfingerQuery {
    resource: String,
}

pub async fn get(
    State(state): State<AppState>,
    Query(WebfingerQuery { resource }): Query<WebfingerQuery>,
) -> impl IntoResponse {
    // Validation
    if !resource.starts_with("acct:") {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid resource format"})),
        )
            .into_response();
    }

    // Get user
    let acct = &resource[5..];
    let parts: Vec<&str> = acct.split("@").collect();
    let username = parts[0];

    let user = queries::user::get_by_username(&state, username).await;

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
        HeaderValue::from_static("application/jrd+json"),
    );
    let json_body = json!({
        "subject": resource,
        "links": [
            {
                "rel": "self",
                "type": "application/activity+json",
                "href": user.ap_url,
            }
        ]
    });

    (json_headers, Json(json_body)).into_response()
}

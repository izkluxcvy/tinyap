mod accept;
mod follow;

use crate::back::init::AppState;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::Value;

pub async fn post(State(state): State<AppState>, Json(activity): Json<Value>) -> impl IntoResponse {
    println!("Received activity:\n{}", activity);

    // Check required fields
    let Some(actor) = activity["actor"].as_str() else {
        return (StatusCode::BAD_REQUEST, "missing actor").into_response();
    };

    let Some(activity_type) = activity["type"].as_str() else {
        return (StatusCode::BAD_REQUEST, "missing type").into_response();
    };

    // Prevent loopback activity
    if actor.contains("localhost")
        || actor.contains("127.0.0.1")
        || actor.contains("[::1]")
        || actor.contains(&state.domain)
    {
        return (StatusCode::BAD_REQUEST, "loopback not allowed").into_response();
    }

    match activity_type {
        "Follow" => follow::follow(&state, &activity).await,
        "Accept" => accept::follow(&state, &activity).await,
        _ => {}
    }

    (StatusCode::OK, "activity received").into_response()
}

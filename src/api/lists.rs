use axum::extract::Json;
use serde_json::{Value, json};

// Placeholder function for toot tui
pub async fn get() -> Json<Value> {
    Json(json!([]))
}

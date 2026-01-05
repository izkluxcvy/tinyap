use crate::api::timeline::{extract_id, timeline_json};
use crate::back::init::AppState;
use crate::back::queries;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde_json::{Value, json};

pub async fn get(State(state): State<AppState>, Path(username): Path<String>) -> Json<Value> {
    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return Json(json!({"error": "User not found"}));
    };

    Json(json!({
        "id": &user.username,
        "username": &user.username,
        "acct": &user.username,
        "display_name": &user.display_name,
        "created_at": &user.created_at,
        "note": &user.bio,
        "followers_count": user.follower_count,
        "following_count": user.following_count,
        "statuses_count": user.note_count,
        "last_status_at": &user.updated_at,
    }))
}

#[derive(serde::Deserialize)]
pub struct StatusesQuery {
    pub limit: Option<i64>,
    pub max_id: Option<i64>,
    pub pinned: Option<bool>,
}

pub async fn get_statuses(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<StatusesQuery>,
) -> Json<Value> {
    // Return empty if pinned is true
    if let Some(pinned) = query.pinned {
        if pinned {
            return Json(json!([]));
        }
    }

    // Extract limit
    let limit = query.limit.unwrap_or(20);
    let limit = if limit > 40 { 40 } else { limit };

    // Extract max_id
    let until = extract_id(&state, query.max_id, "9999").await;

    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return Json(json!({"error": "User not found"}));
    };

    // Get notes by user
    let notes = queries::timeline::get_user(&state, user.id, &until, limit).await;

    let notes_json = timeline_json(&state, notes).await;

    Json(json!(notes_json))
}

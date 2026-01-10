use crate::api::timeline::timeline_json;
use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde_json::{Value, json};

pub fn account_json(
    state: &AppState,
    username: &str,
    display_name: &str,
    created_at: &str,
    bio: &str,
    follower_count: i64,
    following_count: i64,
    note_count: i64,
    updated_at: &str,
) -> Value {
    let avatar_placeholder = format!("https://{}/static/missing.png", state.domain);
    json!({
        "id": username,
        "username": username,
        "acct": username,
        "display_name": display_name,
        "avatar": avatar_placeholder,
        "header": avatar_placeholder,
        "created_at": created_at,
        "note": bio,
        "followers_count": follower_count,
        "following_count": following_count,
        "statuses_count": note_count,
        "last_status_at": updated_at,
        "fields": [],
        "locked": false,
        "emojis": [],
        "url": utils::user_url(&state.domain, username),
        "bot": false,
        "source": {
            "privacy": "public",
            "sensitive": false,
            "note": "",
            "fields": []
        }
    })
}

pub async fn get(State(state): State<AppState>, Path(username): Path<String>) -> Json<Value> {
    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return Json(json!({"error": "User not found"}));
    };

    let account_json = account_json(
        &state,
        &user.username,
        &user.display_name,
        &user.created_at,
        &user.bio,
        user.follower_count,
        user.following_count,
        user.note_count,
        &user.updated_at,
    );

    Json(account_json)
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
    let (until_date, until_id) = utils::extract_until_id(&state, query.max_id).await;

    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return Json(json!({"error": "User not found"}));
    };

    // Get notes by user
    let notes = queries::timeline::get_user(&state, user.id, &until_date, until_id, limit).await;

    let notes_json = timeline_json(&state, notes);

    Json(json!(notes_json))
}

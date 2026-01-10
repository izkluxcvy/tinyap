use crate::api::accounts::account_json;
use crate::back::init::AppState;
use crate::back::queries;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde_json::{Value, json};

pub fn users_json(state: &AppState, users: Vec<queries::follow::FollowUserRecord>) -> Value {
    let users_json: Value = users
        .into_iter()
        .map(|user| {
            account_json(
                state,
                &user.username,
                &user.display_name,
                "9999-01-01T00:00:00Z",
                "",
                0,
                0,
                0,
                "9999-01-01T00:00:00Z",
            )
        })
        .collect();

    users_json
}

#[derive(serde::Deserialize)]
pub struct FollowingQuery {
    pub max_id: Option<String>,
    pub limit: Option<i64>,
}

pub async fn get_following(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<FollowingQuery>,
) -> Json<Value> {
    // extract query parameters
    let max_id = query.max_id.unwrap_or("".to_string());
    let limit = query.limit.unwrap_or(40);
    let limit = if limit > 80 { 80 } else { limit };

    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return Json(json!({"error": "User not found"}));
    };

    // Get following
    let following = queries::follow::get_following(&state, user.id, &max_id, limit).await;

    let following_json = users_json(&state, following);

    Json(following_json)
}

pub async fn get_followers(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<FollowingQuery>,
) -> Json<Value> {
    // extract query parameters
    let max_id = query.max_id.unwrap_or("".to_string());
    let limit = query.limit.unwrap_or(40);
    let limit = if limit > 80 { 80 } else { limit };

    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return Json(json!({"error": "User not found"}));
    };

    // Get followers
    let followers = queries::follow::get_followers(&state, user.id, &max_id, limit).await;

    let followers_json = users_json(&state, followers);

    Json(followers_json)
}

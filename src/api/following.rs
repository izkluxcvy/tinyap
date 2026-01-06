use crate::back::init::AppState;
use crate::back::queries;

use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::{Value, json};

pub fn users_json(users: Vec<queries::follow::FollowUserRecord>) -> Value {
    let users_json: Value = users
        .into_iter()
        .map(|user| {
            json!({
                "id": &user.username,
                "username": &user.username,
                "acct": &user.username,
                "display_name": &user.display_name,
            })
        })
        .collect();

    users_json
}

pub async fn get_following(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Json<Value> {
    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return Json(json!({"error": "User not found"}));
    };

    // Get following
    let following = queries::follow::get_following(&state, user.id).await;

    let following_json = users_json(following);

    Json(following_json)
}

pub async fn get_followers(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Json<Value> {
    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return Json(json!({"error": "User not found"}));
    };

    // Get followers
    let followers = queries::follow::get_followers(&state, user.id).await;

    let followers_json = users_json(followers);

    Json(followers_json)
}

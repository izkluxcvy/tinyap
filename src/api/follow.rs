use crate::api::auth::OAuthUser;
use crate::back::follow;
use crate::back::init::AppState;
use crate::back::queries;

use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::{Value, json};

pub async fn post_follow(
    State(state): State<AppState>,
    Path(username): Path<String>,
    user: OAuthUser,
) -> Json<Value> {
    // Get followee
    let Some(followee) = queries::user::get_by_username(&state, &username).await else {
        return Json(json!({"error": "User not found"}));
    };

    // Follow
    let res = follow::follow(&state, user.id, followee.id).await;
    match res {
        Ok(_) => {
            // When local user
            if followee.is_local == 1 {
                follow::accept(&state, user.id, followee.id).await;

            // When remote user
            } else {
                follow::deliver_follow(&state, user.id, followee.id).await;
            }

            Json(json!({
                "id": &followee.username,
                "following": true,
                "showing_reblogs": true,
                "notifying": true,
            }))
        }
        Err(e) => Json(json!({"error": e})),
    }
}

pub async fn post_unfollow(
    State(state): State<AppState>,
    Path(username): Path<String>,
    user: OAuthUser,
) -> Json<Value> {
    // Get followee
    let Some(followee) = queries::user::get_by_username(&state, &username).await else {
        return Json(json!({"error": "User not found"}));
    };

    // Unfollow
    let res = follow::unfollow(&state, user.id, followee.id).await;
    match res {
        Ok(_) => {
            if followee.is_local == 0 {
                follow::deliver_unfollow(&state, user.id, followee.id).await;
            }
            Json(json!({
                "id": &followee.username,
                "following": false,
                "showing_reblogs": false,
                "notifying": false,
            }))
        }
        Err(e) => Json(json!({"error": e})),
    }
}

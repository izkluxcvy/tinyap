use crate::api::auth::OAuthUser;
use crate::back::init::AppState;
use crate::back::queries;

use axum::{
    Json,
    extract::{Query, State},
};
use serde_json::{Value, json};
use std::collections::HashMap;

pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
    auth_user: OAuthUser,
) -> Json<Value> {
    // Extract id[] or id parameter
    let mut username = "";
    for (key, value) in query.iter() {
        if key == "id[]" || key == "id" {
            username = value;
            break;
        }
    }
    if username == "" {
        return Json(json!({"error": "Missing id parameter"}));
    }

    // Get user
    let Some(user) = queries::user::get_by_username(&state, username).await else {
        return Json(json!({"error": "User not found"}));
    };

    // Check relationships
    let is_following = queries::follow::get(&state, auth_user.id, user.id)
        .await
        .is_some();
    let is_followed_by = queries::follow::get(&state, user.id, auth_user.id)
        .await
        .is_some();

    Json(json!([
        {
            "id": user.username,
            "following": is_following,
            "showing_reblogs": true,
            "notifying": true,
            "followed_by": is_followed_by,
            "blocking": false,
            "blocked_by": false,
            "muting": false,
            "muting_notifications": false,
            "requested": false,
            "domain_blocking": false,
            "endorsed": false
        }
    ]))
}

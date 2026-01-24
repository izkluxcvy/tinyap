use crate::api::auth::OAuthUser;
use crate::back::init::AppState;
use crate::back::queries;

use axum::{
    Json,
    extract::{Query, State},
};
use serde_json::{Value, json};

pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<Vec<(String, String)>>,
    auth_user: OAuthUser,
) -> Json<Value> {
    // Extract id[] or id parameter
    let mut usernames = Vec::new();
    for (key, value) in query.iter() {
        if key == "id[]" || key == "id" {
            usernames.push(value.clone());
        }
    }

    // Get following, followers
    let following = queries::follow::get_following_in(&state, auth_user.id, &usernames).await;
    let followers = queries::follow::get_followers_in(&state, auth_user.id, &usernames).await;

    // Check relationships
    let mut relationships: Vec<Value> = Vec::new();
    for username in usernames {
        let is_following = following.iter().any(|u| u.username == username);
        let is_followed_by = followers.iter().any(|u| u.username == username);

        relationships.push(json!({
            "id": username,
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
        }));
    }

    Json(json!(relationships))
}

use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use serde_json::json;

pub async fn follow(state: &AppState, follower_id: i64, followee_id: i64) -> Result<(), String> {
    // Prevent self-follow
    if follower_id == followee_id {
        return Err("Cannot follow yourself".to_string());
    }

    // Check if already following
    let existing = queries::follow::get(&state, follower_id, followee_id).await;
    if existing.is_some() {
        return Err("Already following or request pending".to_string());
    }

    queries::follow::create(&state, follower_id, followee_id).await;

    Ok(())
}

pub async fn deliver_follow(state: &AppState, follower_id: i64, followee_id: i64) {
    let follower = queries::user::get_by_id(&state, follower_id).await;
    let followee = queries::user::get_by_id(&state, followee_id).await;

    let follow_id = format!("{}#follow-{}", follower.ap_url, utils::gen_unique_id());
    let follow_json = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": follow_id,
        "type": "Follow",
        "actor": follower.ap_url,
        "object": followee.ap_url,
    });
    let json_body = follow_json.to_string();

    let private_key = follower.private_key.unwrap();
    utils::signed_deliver(
        &state,
        &follower.ap_url,
        &private_key,
        &followee.inbox_url,
        &json_body,
    )
    .await;
}

pub async fn approve(state: &AppState, follower_id: i64, followee_id: i64) {
    queries::follow::approve(&state, follower_id, followee_id).await;
}

pub async fn unfollow(state: &AppState, follower_id: i64, followee_id: i64) -> Result<(), String> {
    // Check if following
    let existing = queries::follow::get(&state, follower_id, followee_id).await;
    if existing.is_none() {
        return Err("Not following".to_string());
    }

    queries::follow::delete(&state, follower_id, followee_id).await;
    Ok(())
}

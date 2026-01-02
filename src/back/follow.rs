use crate::back::init::AppState;
use crate::back::notification;
use crate::back::queries;
use crate::back::utils;

use serde_json::json;

pub async fn follow(state: &AppState, follower_id: i64, followee_id: i64) -> Result<(), String> {
    // Prevent self-follow
    if follower_id == followee_id {
        return Err("Cannot follow yourself".to_string());
    }

    // Check if already following
    let existing = queries::follow::get(state, follower_id, followee_id).await;
    if existing.is_some() {
        return Err("Already following or request pending".to_string());
    }

    // Follow
    queries::follow::create(state, follower_id, followee_id).await;

    // Add notification
    notification::add(
        state,
        notification::EventType::Follow,
        follower_id,
        followee_id,
        None,
    )
    .await;

    // Increment following and follower counts
    queries::user::increment_following_count(state, follower_id).await;
    queries::user::increment_follower_count(state, followee_id).await;

    Ok(())
}

pub async fn deliver_follow(state: &AppState, follower_id: i64, followee_id: i64) {
    let follower = queries::user::get_by_id(state, follower_id).await;
    let followee = queries::user::get_by_id(state, followee_id).await;

    let follow_id = format!("{}#follow-{}", follower.ap_url, utils::gen_unique_id());
    let follow_activity = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": follow_id,
        "type": "Follow",
        "actor": follower.ap_url,
        "object": followee.ap_url,
    });
    let json_body = follow_activity.to_string();

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

pub async fn accept(state: &AppState, follower_id: i64, followee_id: i64) {
    queries::follow::accept(state, follower_id, followee_id).await;
}

pub async fn unfollow(state: &AppState, follower_id: i64, followee_id: i64) -> Result<(), String> {
    // Check if following
    let existing = queries::follow::get(state, follower_id, followee_id).await;
    if existing.is_none() {
        return Err("Not following".to_string());
    }

    // Unfollow
    queries::follow::delete(state, follower_id, followee_id).await;

    // Decrement following and follower counts
    queries::user::decrement_following_count(state, follower_id).await;
    queries::user::decrement_follower_count(state, followee_id).await;
    Ok(())
}

pub async fn deliver_unfollow(state: &AppState, follower_id: i64, followee_id: i64) {
    let follower = queries::user::get_by_id(state, follower_id).await;
    let followee = queries::user::get_by_id(state, followee_id).await;

    let undo_id = format!("{}#undo-{}", follower.ap_url, utils::gen_unique_id());
    let unfollow_activity = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": undo_id,
        "type": "Undo",
        "actor": follower.ap_url,
        "object": {
            "type": "Follow",
            "actor": follower.ap_url,
            "object": followee.ap_url,
        }
    });
    let json_body = unfollow_activity.to_string();

    let private_key = follower.private_key.unwrap();
    utils::signed_deliver(
        state,
        &follower.ap_url,
        &private_key,
        &followee.inbox_url,
        &json_body,
    )
    .await;
}

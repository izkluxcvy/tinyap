use crate::back::init::AppState;
use crate::back::queries;

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

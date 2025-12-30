use crate::back::follow;
use crate::back::init::AppState;
use crate::back::like;
use crate::back::queries;

use serde_json::Value;

pub async fn follow(state: &AppState, activity: &Value) {
    // Extract actor and object
    let Some(follower_ap_url) = activity["object"]["actor"].as_str() else {
        return;
    };
    let Some(followee_ap_url) = activity["object"]["object"].as_str() else {
        return;
    };

    // Get users
    let Some(follower) = queries::user::get_by_ap_url(state, follower_ap_url).await else {
        return;
    };
    let Some(followee) = queries::user::get_by_ap_url(state, followee_ap_url).await else {
        return;
    };

    // Unfollow
    let _ = follow::unfollow(state, follower.id, followee.id).await;
}

pub async fn like(state: &AppState, activity: &Value) {
    // Extract actor and object
    let Some(liker_ap_url) = activity["object"]["actor"].as_str() else {
        return;
    };
    let Some(note_ap_url) = activity["object"]["object"].as_str() else {
        return;
    };

    // Get user
    let Some(liker) = queries::user::get_by_ap_url(state, liker_ap_url).await else {
        return;
    };

    // Get note
    let Some(note) = queries::note::get_by_ap_url(state, note_ap_url).await else {
        return;
    };

    // Unlike
    like::unlike(state, liker.id, note.id).await;
}

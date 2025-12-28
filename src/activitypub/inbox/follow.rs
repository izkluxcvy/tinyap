use crate::back::follow;
use crate::back::init::AppState;
use crate::back::queries;
use crate::back::user;
use crate::back::utils;

use serde_json::{Value, json};

pub async fn follow(state: &AppState, activity: &Value) {
    // Extract actor and object
    let Some(follower_ap_url) = activity["actor"].as_str() else {
        return;
    };
    let Some(followee_ap_url) = activity["object"].as_str() else {
        return;
    };

    // Get followee = local user
    let Some(followee) = queries::user::get_by_ap_url(&state, followee_ap_url).await else {
        return;
    };
    let private_key = followee.private_key.unwrap();

    // Create locally-stored remote user if not exists
    let follower = {
        if let Some(follower) = queries::user::get_by_ap_url(&state, follower_ap_url).await {
            follower
        } else {
            let res =
                user::add_remote(&state, &followee.ap_url, &private_key, follower_ap_url).await;
            if res.is_err() {
                return;
            }

            queries::user::get_by_ap_url(&state, follower_ap_url)
                .await
                .unwrap()
        }
    };

    // Follow
    let res = follow::follow(&state, follower.id, followee.id).await;
    if res.is_err() {
        return;
    }
    follow::accept(&state, follower.id, followee.id).await;

    // Return Accept activity
    let accept_id = format!("{}#accept-{}", follower.ap_url, utils::gen_unique_id());
    let accept_activity = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": accept_id,
        "type": "Accept",
        "actor": followee.ap_url,
        "object": activity,
    });
    let json_body = accept_activity.to_string();

    utils::signed_deliver(
        &state,
        &followee.ap_url,
        &private_key,
        &follower.inbox_url,
        &json_body,
    )
    .await;
}

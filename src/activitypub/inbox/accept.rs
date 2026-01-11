use crate::back::follow;
use crate::back::init::AppState;
use crate::back::queries;

use serde_json::Value;

pub async fn follow(state: &AppState, activity: &Value) {
    // Extract actor and object
    // actor:  remote user who accepts follow request
    // object: local user who sent follow request
    let Some(actor_ap_url) = activity["actor"].as_str() else {
        return;
    };

    let object_ap_url = if let Some(obj) = activity["object"].as_str() {
        obj
    } else if let Some(obj) = activity["object"]["actor"].as_str() {
        obj
    } else {
        return;
    };

    // Get user
    let Some(actor) = queries::user::get_by_ap_url(state, actor_ap_url).await else {
        return;
    };

    let Some(object) = queries::user::get_by_ap_url(state, object_ap_url).await else {
        return;
    };

    // Accept
    let existing = queries::follow::get(state, object.id, actor.id).await;
    if existing.is_none() {
        follow::accept(&state, object.id, actor.id).await;
    }
}

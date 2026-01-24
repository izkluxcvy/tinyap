use crate::back::init::AppState;
use crate::back::like;
use crate::back::queries;
use crate::back::user;

use serde_json::Value;

pub async fn like(state: &AppState, activity: &Value) {
    // Extract actor and object
    let Some(liker_ap_url) = activity["actor"].as_str() else {
        return;
    };
    let Some(note_ap_url) = activity["object"].as_str() else {
        return;
    };

    // Create locally-stored remote user if not exists
    let actor = {
        if let Some(actor) = queries::user::get_by_ap_url(state, liker_ap_url).await {
            actor
        } else {
            let res = user::add_remote(state, liker_ap_url).await;
            if res.is_err() {
                return;
            }

            queries::user::get_by_ap_url(state, liker_ap_url)
                .await
                .unwrap()
        }
    };

    // Get note
    let Some(note) = queries::note::get_by_ap_url(state, note_ap_url).await else {
        return;
    };

    // Like
    let _ = like::like(state, actor.id, note.id).await;
}

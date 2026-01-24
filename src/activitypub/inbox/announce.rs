use crate::back::boost;
use crate::back::init::AppState;
use crate::back::note;
use crate::back::queries;

use serde_json::Value;

pub async fn announce(state: &AppState, activity: &Value) {
    // Extract actor and object
    let Some(booster_ap_url) = activity["actor"].as_str() else {
        return;
    };
    let Some(note_ap_url) = activity["object"].as_str() else {
        return;
    };

    // Create or booster if not exists
    let booster = if let Some(booster) = queries::user::get_by_ap_url(state, booster_ap_url).await {
        booster
    } else {
        let _ = note::add_remote(state, booster_ap_url).await;
        queries::user::get_by_ap_url(state, booster_ap_url)
            .await
            .unwrap()
    };

    // Create boosted note recursively if not exists
    let note = if let Some(note) = queries::note::get_by_ap_url(state, note_ap_url).await {
        note
    } else {
        let _ = note::add_remote(state, note_ap_url).await;
        let Some(note) = queries::note::get_by_ap_url(state, note_ap_url).await else {
            return;
        };
        note
    };

    // Boost
    let _ = boost::boost(state, booster.id, note.id).await;
}

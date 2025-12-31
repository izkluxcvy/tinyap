use crate::back::init::AppState;
use crate::back::note;
use crate::back::queries;

use serde_json::Value;

pub async fn note(state: &AppState, activity: &Value) {
    // Extract object
    let note_ap_url = if let Some(obj) = activity["object"].as_str() {
        obj
    } else if let Some(obj) = activity["object"]["id"].as_str() {
        obj
    } else {
        return;
    };

    // Get note
    let Some(note) = queries::note::get_by_ap_url(state, note_ap_url).await else {
        return;
    };

    // Delete
    note::delete(state, note.id).await;
}

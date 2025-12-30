use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use serde_json::json;

pub async fn boost(state: &AppState, user_id: i64, note_id: i64) -> Result<(), String> {
    // Check if already boosted
    let existing = queries::boost::get(state, user_id, note_id).await;
    if existing.is_some() {
        return Err("Already boosted".to_string());
    }

    // Get note
    let Some(note) = queries::note::get_by_id(state, note_id).await else {
        return Err("Note not found".to_string());
    };

    // Get author
    let author = queries::user::get_by_id(state, note.author_id).await;

    // Boost
    let boost_id = utils::gen_unique_id();
    let ap_url = utils::local_note_ap_url(&state.domain, boost_id);
    let date_now = utils::date_now();
    queries::boost::create(
        state,
        boost_id,
        &ap_url,
        user_id,
        note.id,
        &author.username,
        &note.created_at,
        &note.content,
        note.attachments,
        note.parent_id,
        note.parent_author_username,
        &date_now,
    )
    .await;

    // Increment boost count
    queries::note::increment_boost_count(state, note.id).await;

    Ok(())
}

pub async fn deliver_boost(state: &AppState, user_id: i64, note_id: i64) {
    // Get user
    let user = queries::user::get_by_id(state, user_id).await;

    // Get note
    let Some(note) = queries::note::get_by_id(state, note_id).await else {
        return;
    };

    // Get author
    let author = queries::user::get_by_id(state, note.author_id).await;

    // Deliver
    let announce_id = format!("{}#announce-{}", user.ap_url, utils::gen_unique_id());
    let announce_activity = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": announce_id,
        "type": "Announce",
        "actor": user.ap_url,
        "object": note.ap_url,
        "to": ["https://www.w3.org/ns/activitystreams#Public"]
    });
    let json_body = announce_activity.to_string();

    utils::deliver_to_followers(state, user_id, Some(author.inbox_url), &json_body).await;
}

pub async fn unboost(state: &AppState, user_id: i64, note_id: i64) -> Result<(), String> {
    // Unboost
    queries::boost::delete(state, user_id, note_id).await;

    // Decrement boost count
    queries::note::decrement_boost_count(state, note_id).await;

    Ok(())
}

pub async fn deliver_unboost(state: &AppState, user_id: i64, note_id: i64) {
    // Get user
    let user = queries::user::get_by_id(state, user_id).await;

    // Get note
    let Some(note) = queries::note::get_by_id(state, note_id).await else {
        return;
    };

    // Get author
    let author = queries::user::get_by_id(state, note.author_id).await;

    // Deliver
    let undo_id = format!("{}#undo-{}", user.ap_url, utils::gen_unique_id());
    let unboost_activity = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": undo_id,
        "type": "Undo",
        "actor": user.ap_url,
        "object": {
            "type": "Announce",
            "actor": user.ap_url,
            "object": note.ap_url,
        }
    });
    let json_body = unboost_activity.to_string();

    utils::deliver_to_followers(state, user_id, Some(author.inbox_url), &json_body).await;
}

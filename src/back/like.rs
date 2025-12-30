use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use serde_json::json;

pub async fn like(state: &AppState, user_id: i64, note_id: i64) -> Result<(), String> {
    // Check if already liked
    let existing = queries::like::get(state, user_id, note_id).await;
    if existing.is_some() {
        return Err("Already liked".to_string());
    }

    // Like
    queries::like::create(state, user_id, note_id).await;

    // Increment like count
    queries::note::increment_like_count(state, note_id).await;
    Ok(())
}

pub async fn deliver_like(state: &AppState, user_id: i64, note_id: i64) {
    let user = queries::user::get_by_id(state, user_id).await;
    let note = queries::note::get_by_id(state, note_id).await.unwrap();
    let author = queries::user::get_by_id(state, note.author_id).await;

    let like_id = format!("{}#like-{}", user.ap_url, utils::gen_unique_id());
    let like_activity = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": like_id,
        "type": "Like",
        "actor": user.ap_url,
        "object": note.ap_url,
    });
    let json_body = like_activity.to_string();

    let private_key = user.private_key.unwrap();
    utils::signed_deliver(
        &state,
        &user.ap_url,
        &private_key,
        &author.inbox_url,
        &json_body,
    )
    .await;
}

pub async fn unlike(state: &AppState, user_id: i64, note_id: i64) {
    // Unlike
    queries::like::delete(&state, user_id, note_id).await;

    // Decrement like count
    queries::note::decrement_like_count(state, note_id).await;
}

pub async fn deliver_unlilke(state: &AppState, user_id: i64, note_id: i64) {
    let user = queries::user::get_by_id(state, user_id).await;
    let note = queries::note::get_by_id(state, note_id).await.unwrap();
    let author = queries::user::get_by_id(state, note.author_id).await;

    let undo_id = format!("{}#undo-{}", user.ap_url, utils::gen_unique_id());
    let unlike_activity = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": undo_id,
        "type": "Undo",
        "actor": user.ap_url,
        "object": {
            "type": "Like",
            "actor": user.ap_url,
            "object": note.ap_url,
        }
    });
    let json_body = unlike_activity.to_string();

    let private_key = user.private_key.unwrap();
    utils::signed_deliver(
        &state,
        &user.ap_url,
        &private_key,
        &author.inbox_url,
        &json_body,
    )
    .await;
}

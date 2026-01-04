use crate::api::auth::OAuthUser;
use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::{Value, json};

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: OAuthUser,
) -> Json<Value> {
    let Some(note) = queries::note::get_with_author_by_id(&state, id).await else {
        return Json(json!({
            "error": "Note not found"
        }));
    };

    let attachments = utils::attachments_to_value(&state, &note.attachments);

    // Check is_liked, is_boosted
    let is_liked = if let Some(_like) = queries::like::get(&state, user.id, note.id).await {
        true
    } else {
        false
    };

    let is_boosted = if let Some(_boost) = queries::boost::get(&state, user.id, note.id).await {
        true
    } else {
        false
    };

    Json(json!({
        "id": note.id,
        "created_at": &note.created_at,
        "in_reply_to_id": note.parent_id,
        "visibility": "public",
        "reblogs_count": note.boost_count,
        "favourites_count": note.like_count,
        "content": &note.content,
        "account": {
            "id": note.author_id,
            "username": &note.username,
            "acct": &note.username,
            "display_name": &note.display_name,
        },
        "media_attachments": attachments,
        "favourited": is_liked,
        "reblogged": is_boosted,
    }))
}

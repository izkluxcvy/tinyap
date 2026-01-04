use crate::api::auth::OAuthUser;
use crate::back::init::AppState;
use crate::back::like;
use crate::back::queries;

use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::{Value, json};

pub async fn post_favourite(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: OAuthUser,
) -> Json<Value> {
    // Get note
    let Some(note) = queries::note::get_with_author_by_id(&state, id).await else {
        return Json(json!({
            "error": "Note not found"
        }));
    };

    // Get author
    let author = queries::user::get_by_id(&state, note.author_id).await;

    // Like
    let res = like::like(&state, user.id, id).await;
    if let Err(e) = res {
        return Json(json!({
            "error": e
        }));
    };

    // Deliver like
    if author.is_local == 0 {
        like::deliver_like(&state, user.id, id).await;
    }

    Json(json!({
        "id": note.id,
        "created_at": &note.created_at,
        "in_reply_to_id": note.parent_id,
        "visibility": "public",
        "reblogs_count": note.boost_count,
        "favourites_count": note.like_count + 1,
        "content": &note.content,
        "account": {
            "id": note.author_id,
            "username": &note.username,
            "acct": &note.username,
            "display_name": &note.display_name,
        },
        "favourited": true,
    }))
}

pub async fn post_unfavourite(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: OAuthUser,
) -> Json<Value> {
    // Get note
    let Some(note) = queries::note::get_with_author_by_id(&state, id).await else {
        return Json(json!({
            "error": "Note not found"
        }));
    };

    // Get author
    let author = queries::user::get_by_id(&state, note.author_id).await;

    // Unlike
    like::unlike(&state, user.id, id).await;

    // Deliver unlike
    if author.is_local == 0 {
        like::deliver_unlilke(&state, user.id, id).await;
    }

    Json(json!({
        "id": note.id,
        "created_at": &note.created_at,
        "in_reply_to_id": note.parent_id,
        "visibility": "public",
        "reblogs_count": note.boost_count,
        "favourites_count": note.like_count - 1,
        "content": &note.content,
        "account": {
            "id": note.author_id,
            "username": &note.username,
            "acct": &note.username,
            "display_name": &note.display_name,
        },
        "favourited": false,
    }))
}

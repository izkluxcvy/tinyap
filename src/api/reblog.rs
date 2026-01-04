use crate::api::auth::OAuthUser;
use crate::back::boost;
use crate::back::init::AppState;
use crate::back::queries;

use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::{Value, json};

pub async fn post_reblog(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: OAuthUser,
) -> Json<Value> {
    // Get note
    let Some(note) = queries::note::get_with_author_by_id(&state, id).await else {
        return Json(json!({"error": "Note not found"}));
    };

    // Boost
    let res = boost::boost(&state, user.id, id).await;
    if let Err(e) = res {
        return Json(json!({"error": e}));
    };

    // Deliver to followers
    boost::deliver_boost(&state, user.id, id).await;

    Json(json!({
        "id": note.id,
        "created_at": &note.created_at,
        "in_reply_to_id": note.parent_id,
        "visibility": "public",
        "reblogs_count": note.boost_count + 1,
        "favourites_count": note.like_count,
        "content": &note.content,
        "account": {
            "id": note.author_id,
            "username": &note.username,
            "acct": &note.username,
            "display_name": &note.display_name,
        },
        "reblogged": true,
    }))
}

pub async fn post_unreblog(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: OAuthUser,
) -> Json<Value> {
    // Get note
    let Some(note) = queries::note::get_with_author_by_id(&state, id).await else {
        return Json(json!({"error": "Note not found"}));
    };

    // Unboost
    let res = boost::unboost(&state, user.id, id).await;
    if let Err(e) = res {
        return Json(json!({"error": e}));
    };

    // Deliver to followers
    boost::deliver_unboost(&state, user.id, id).await;

    Json(json!({
        "id": note.id,
        "created_at": &note.created_at,
        "in_reply_to_id": note.parent_id,
        "visibility": "public",
        "reblogs_count": note.boost_count - 1,
        "favourites_count": note.like_count,
        "content": &note.content,
        "account": {
            "id": note.author_id,
            "username": &note.username,
            "acct": &note.username,
            "display_name": &note.display_name,
        },
        "reblogged": false,
    }))
}

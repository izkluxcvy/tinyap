use crate::api::accounts::account_json;
use crate::api::auth::OAuthUser;
use crate::api::statuses::status_json;
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

    let account_json = account_json(
        &state,
        &note.username,
        &note.display_name,
        &note.created_at,
        "",
        0,
        0,
        0,
        &note.created_at,
    );
    let status_json = status_json(
        &state,
        note.id,
        &note.username,
        None,
        None,
        None,
        &note.content,
        &account_json,
        &note.created_at,
        &vec![],
        note.like_count + 1,
        note.boost_count,
        false,
        false,
        note.parent_id,
        None,
    );

    Json(status_json)
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

    let account_json = account_json(
        &state,
        &note.username,
        &note.display_name,
        &note.created_at,
        "",
        0,
        0,
        0,
        &note.created_at,
    );
    let status_json = status_json(
        &state,
        note.id,
        &note.username,
        None,
        None,
        None,
        &note.content,
        &account_json,
        &note.created_at,
        &vec![],
        note.like_count - 1,
        note.boost_count,
        false,
        false,
        note.parent_id,
        None,
    );

    Json(status_json)
}

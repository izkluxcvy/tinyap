use crate::api::accounts::account_json;
use crate::api::auth::OAuthUser;
use crate::api::statuses::status_json;
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

    let account_json = account_json(
        &state,
        &note.username,
        &note.display_name,
        &note.created_at,
        &note.content,
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
        note.like_count,
        note.boost_count,
        false,
        true,
        note.parent_id,
        None,
    );

    Json(status_json)
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
        note.like_count,
        note.boost_count,
        false,
        false,
        note.parent_id,
        None,
    );

    Json(status_json)
}

use crate::api::auth::OAuthUser;
use crate::back::init::AppState;
use crate::back::note;
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
            "id": &note.username,
            "username": &note.username,
            "acct": &note.username,
            "display_name": &note.display_name,
        },
        "media_attachments": attachments,
        "favourited": is_liked,
        "reblogged": is_boosted,
    }))
}

#[derive(serde::Deserialize)]
pub struct PostStatusRequest {
    pub status: String,
    pub in_reply_to_id: Option<i64>,
}

pub async fn post(
    State(state): State<AppState>,
    user: OAuthUser,
    Json(req): Json<PostStatusRequest>,
) -> Json<Value> {
    let user = queries::user::get_by_id(&state, user.id).await;
    let id = utils::gen_unique_id();
    let ap_url = utils::local_note_ap_url(&state.domain, id);
    let created_at = utils::date_now();

    // in_reply_to handling
    let parent_author_username = if let Some(parent_id) = req.in_reply_to_id {
        let Some(parent) = queries::note::get_by_id(&state, parent_id).await else {
            return Json(json!({
                "error": "Parent note not found"
            }));
        };
        let parent_author = queries::user::get_by_id(&state, parent.author_id).await;
        Some(parent_author.username)
    } else {
        None
    };

    // Create note
    let res = note::add(
        &state,
        id,
        &ap_url,
        user.id,
        &req.status,
        None,
        req.in_reply_to_id,
        parent_author_username,
        &created_at,
        1, // is_public
    )
    .await;

    if let Err(e) = res {
        println!("Error creating note: {}", e);
        return Json(json!({
            "error": "Something went wrong"
        }));
    }

    // Deliver to followers and parent
    note::deliver_create(&state, id).await;

    Json(json!({
        "id": id,
        "created_at": created_at,
        "in_reply_to_id": req.in_reply_to_id,
        "visibility": "public",
        "reblogs_count": 0,
        "favourites_count": 0,
        "content": req.status,
        "account": {
            "id": &user.username,
            "username": &user.username,
            "acct": &user.username,
            "display_name": &user.display_name,
        },
        "media_attachments": [],
    }))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    user: OAuthUser,
) -> Json<Value> {
    // Get user
    let user = queries::user::get_by_id(&state, user.id).await;

    // Get note
    let Some(note) = queries::note::get_with_author_by_id(&state, id).await else {
        return Json(json!({"error": "Note not found"}));
    };
    if note.author_id != user.id {
        return Json(json!({"error": "Unauthorized"}));
    }

    // Deliver delete activity
    note::deliver_delete(&state, id).await;

    // Delete
    note::delete(&state, id).await;

    Json(json!({
        "id": note.id,
        "created_at": &note.created_at,
        "in_reply_to_id": note.parent_id,
        "visibility": "public",
        "reblogs_count": note.boost_count,
        "favourites_count": note.like_count,
        "content": &note.content,
        "account": {
            "id": &note.username,
            "username": &user.username,
            "acct": &user.username,
            "display_name": &user.display_name,
        },
        "media_attachments": [],
    }))
}

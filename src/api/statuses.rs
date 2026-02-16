use crate::api::accounts::account_json;
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

pub fn status_json(
    state: &AppState,
    id: i64,
    author_username: &str,
    boosted_id: Option<i64>,
    boosted_created_at: Option<String>,
    boosted_account_json: Option<Value>,
    content: &str,
    account_json: &Value,
    created_at: &str,
    attachments: &Vec<Value>,
    like_count: i64,
    boost_count: i64,
    is_liked: bool,
    is_boosted: bool,
    parent_id: Option<i64>,
    parent_author_username: Option<String>,
) -> Value {
    let parent_id_string = parent_id.map(|id| id.to_string());
    if let Some(boosted_id) = boosted_id {
        let reblog_json = json!({
            "id": boosted_id.to_string(),
            "content": content,
            "account": boosted_account_json,
            "created_at": boosted_created_at,
            "media_attachments": attachments,
            "mentions": [],
            "replies_count": 0,
            "favourites_count": 0,
            "reblogs_count": 0,
            "favourited": false,
            "reblogged": false,
            "in_reply_to_id": null,
            "in_reply_to_account_id": null,
            "visibility": "public",
            "emojis": [],
            "uri": utils::local_note_ap_url(&state.domain, boosted_id),
            "url": utils::note_url(&state.domain, author_username, boosted_id),
            "sensitive": false,
            "spoiler_text": "",
            "tags": [],
            "filtered": [],
            "reblog": null,
        });
        json!({
            "id": id.to_string(),
            "content": "",
            "account": account_json,
            "created_at": created_at,
            "media_attachments": [],
            "mentions": [],
            "replies_count": 0,
            "favourites_count": like_count,
            "reblogs_count": boost_count,
            "favourited": is_liked,
            "reblogged": is_boosted,
            "in_reply_to_id": parent_id_string,
            "in_reply_to_account_id": parent_author_username,
            "visibility": "public",
            "emojis": [],
            "uri": utils::local_note_ap_url(&state.domain, id),
            "url": utils::note_url(&state.domain, author_username, id),
            "sensitive": false,
            "spoiler_text": "",
            "tags": [],
            "filtered": [],
            "reblog": reblog_json,
        })
    } else {
        json!({
            "id": id.to_string(),
            "content": content,
            "account": account_json,
            "created_at": created_at,
            "media_attachments": attachments,
            "mentions": [],
            "replies_count": 0,
            "favourites_count": like_count,
            "reblogs_count": boost_count,
            "favourited": is_liked,
            "reblogged": is_boosted,
            "in_reply_to_id": parent_id_string,
            "in_reply_to_account_id": parent_author_username,
            "visibility": "public",
            "emojis": [],
            "uri": utils::local_note_ap_url(&state.domain, id),
            "url": utils::note_url(&state.domain, author_username, id),
            "sensitive": false,
            "spoiler_text": "",
            "tags": [],
            "filtered": [],
            "reblog": null,
        })
    }
}

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
    let is_liked = matches!(
        queries::like::get(&state, user.id, note.id).await,
        Some(_like)
    );

    let is_boosted = matches!(
        queries::boost::get(&state, user.id, note.id).await,
        Some(_boost)
    );

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
        &attachments,
        note.like_count,
        note.boost_count,
        is_liked,
        is_boosted,
        note.parent_id,
        note.parent_author_username,
    );

    Json(status_json)
}

#[derive(serde::Deserialize)]
pub struct PostStatusRequest {
    pub status: String,
    pub in_reply_to_id: Option<String>,
}

pub async fn post(
    State(state): State<AppState>,
    user: OAuthUser,
    Json(req): Json<PostStatusRequest>,
) -> Json<Value> {
    // Option<String> to Option<i64>
    let in_reply_to_id = req
        .in_reply_to_id
        .as_ref()
        .and_then(|id_str| id_str.parse::<i64>().ok());

    let user = queries::user::get_by_id(&state, user.id).await;
    let id = utils::gen_unique_id();
    let ap_url = utils::local_note_ap_url(&state.domain, id);
    let created_at = utils::date_now();

    // in_reply_to handling
    let parent_author_username = if let Some(parent_id) = in_reply_to_id {
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
        in_reply_to_id,
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

    let account_json = account_json(
        &state,
        &user.username,
        &user.display_name,
        &user.created_at,
        &user.bio,
        user.follower_count,
        user.following_count,
        user.note_count,
        &user.updated_at,
    );
    let status_json = status_json(
        &state,
        id,
        &user.username,
        None,
        None,
        None,
        &req.status,
        &account_json,
        &created_at,
        &vec![],
        0,
        0,
        false,
        false,
        in_reply_to_id,
        None,
    );

    Json(status_json)
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

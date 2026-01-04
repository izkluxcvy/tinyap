use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::{Value, json};

pub async fn get(State(state): State<AppState>, Path(id): Path<i64>) -> Json<Value> {
    let Some(note) = queries::note::get_with_author_by_id(&state, id).await else {
        return Json(json!({
            "error": "Note not found"
        }));
    };

    let attachments = if let Some(attachments) = note.attachments {
        if attachments.is_empty() {
            vec![]
        } else {
            let mut ret: Vec<Value> = vec![];
            for url in attachments.split("\n") {
                if url.is_empty() {
                    continue;
                }
                ret.push(json!({
                    "type": "image",
                    "url": utils::strip_content(&state, url),
                }));
            }
            ret
        }
    } else {
        vec![]
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
    }))
}

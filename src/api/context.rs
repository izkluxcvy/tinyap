use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::{Value, json};

pub async fn get(State(state): State<AppState>, Path(id): Path<i64>) -> Json<Value> {
    let Some(note) = queries::note::get_by_id(&state, id).await else {
        return Json(json!({
            "error": "Note not found"
        }));
    };

    // Get ancestor (parent)
    let ancestor = if let Some(parent_id) = note.parent_id {
        queries::note::get_with_author_by_id(&state, parent_id).await
    } else {
        None
    };

    let ancestors_json = if let Some(ancestor) = ancestor {
        let attachments = utils::attachments_to_value(&state, &ancestor.attachments);
        let parent_id_string = ancestor.parent_id.map(|id| id.to_string());
        json!([{
            "id": &ancestor.id.to_string(),
            "created_at": &ancestor.created_at,
            "in_reply_to_id": parent_id_string,
            "visibility": "public",
            "reblogs_count": ancestor.boost_count,
            "favourites_count": ancestor.like_count,
            "content": &ancestor.content,
            "account": {
                "id": &ancestor.username,
                "username": &ancestor.username,
                "acct": &ancestor.username,
                "display_name": &ancestor.display_name,
            },
            "media_attachments": attachments,
        }])
    } else {
        json!([])
    };

    // Get descendants (replies)
    let descendants = queries::note::get_replies_by_parent_id(&state, note.id).await;

    let descendants_json: Value = descendants
        .into_iter()
        .map(|descendant| {
            let attachments = utils::attachments_to_value(&state, &descendant.attachments);
            let parent_id_string = descendant.parent_id.map(|id| id.to_string());
            json!({
                "id": &descendant.id.to_string(),
                "created_at": &descendant.created_at,
                "in_reply_to_id": parent_id_string,
                "visibility": "public",
                "reblogs_count": descendant.boost_count,
                "favourites_count": descendant.like_count,
                "content": &descendant.content,
                "account": {
                    "id": &descendant.username,
                    "username": &descendant.username,
                    "acct": &descendant.username,
                    "display_name": &descendant.display_name,
                },
                "media_attachments": attachments,
            })
        })
        .collect();

    Json(json!({
        "ancestors": ancestors_json,
        "descendants": descendants_json,
    }))
}

use crate::api::accounts::account_json;
use crate::api::statuses::status_json;
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
        let account_json = account_json(
            &state,
            &ancestor.username,
            &ancestor.display_name,
            &ancestor.created_at,
            "",
            0,
            0,
            0,
            &ancestor.created_at,
        );
        let status_json = status_json(
            &state,
            ancestor.id,
            &ancestor.username,
            None,
            None,
            None,
            &ancestor.content,
            &account_json,
            &ancestor.created_at,
            &attachments,
            ancestor.like_count,
            ancestor.boost_count,
            false,
            false,
            ancestor.parent_id,
            ancestor.parent_author_username,
        );

        json!([status_json])
    } else {
        json!([])
    };

    // Get descendants (replies)
    let descendants = queries::note::get_replies_by_parent_id(&state, note.id).await;

    let descendants_json: Value = descendants
        .into_iter()
        .map(|descendant| {
            let attachments = utils::attachments_to_value(&state, &descendant.attachments);
            let account_json = account_json(
                &state,
                &descendant.username,
                &descendant.display_name,
                &descendant.created_at,
                "",
                0,
                0,
                0,
                &descendant.created_at,
            );
            status_json(
                &state,
                descendant.id,
                &descendant.username,
                None,
                None,
                None,
                &descendant.content,
                &account_json,
                &descendant.created_at,
                &attachments,
                descendant.like_count,
                descendant.boost_count,
                false,
                false,
                descendant.parent_id,
                descendant.parent_author_username,
            )
        })
        .collect();

    Json(json!({
        "ancestors": ancestors_json,
        "descendants": descendants_json,
    }))
}

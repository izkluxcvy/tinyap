use crate::api::auth::OAuthUser;
use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    extract::{Query, State},
};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
pub struct TimelineQuery {
    pub limit: Option<i64>,
    pub since_id: Option<i64>,
    pub max_id: Option<i64>,
    pub local: Option<bool>,
}

pub async fn extract_limit(limit: Option<i64>) -> i64 {
    let limit = limit.unwrap_or(20);
    if limit > 40 { 40 } else { limit }
}

pub async fn extract_id(state: &AppState, id: Option<i64>, default: &str) -> String {
    if let Some(id) = id {
        let note = queries::note::get_by_id(&state, id).await;
        if let Some(note) = note {
            note.created_at
        } else {
            default.to_string()
        }
    } else {
        default.to_string()
    }
}

pub async fn timeline_json(
    state: &AppState,
    notes: Vec<queries::note::NoteWithAuthorRecord>,
) -> Value {
    let notes_json: Value = notes
        .into_iter()
        .map(|note| {
            let attachments = utils::attachments_to_value(state, &note.attachments);

            if note.boosted_id.is_some() {
                json!({
                    "id": note.id.to_string(),
                    "created_at": &note.created_at,
                    "in_reply_to_id": note.parent_id,
                    "visibility": "public",
                    "reblogs_count": 0,
                    "favourites_count": 0,
                    "content": "",
                    "account": {
                        "id": &note.username,
                        "username": &note.username,
                        "acct": &note.username,
                        "display_name": &note.display_name,
                    },
                    "media_attachments": [],
                    "reblog": {
                        "id": note.boosted_id,
                        "created_at": &note.boosted_created_at,
                        "in_reply_to_id": null,
                        "visibility": "public",
                        "reblogs_count": 0,
                        "favourites_count": 0,
                        "content": &note.content,
                        "account": {
                            "id": &note.boosted_username,
                            "username": &note.boosted_username,
                            "acct": &note.boosted_username,
                            "display_name": &note.boosted_username,
                        },
                        "media_attachments": attachments,
                    }
                })
            } else {
                json!({
                    "id": note.id.to_string(),
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
                })
            }
        })
        .collect();

    notes_json
}

pub async fn get_home(
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
    user: OAuthUser,
) -> Json<Value> {
    let limit = extract_limit(query.limit).await;

    let notes = if let Some(max_id) = query.max_id {
        let until = extract_id(&state, Some(max_id), "9999").await;
        queries::timeline::get_home(&state, user.id, &until, limit).await
    } else {
        let since = extract_id(&state, query.since_id, "0").await;
        queries::timeline::get_home_since(&state, user.id, &since, limit).await
    };

    let notes_json = timeline_json(&state, notes).await;

    Json(notes_json)
}

pub async fn get_public(
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
) -> Json<Value> {
    let limit = extract_limit(query.limit).await;

    let notes = if query.local.unwrap_or(false) {
        if let Some(max_id) = query.max_id {
            let until = extract_id(&state, Some(max_id), "9999").await;
            queries::timeline::get_local(&state, &until, limit).await
        } else {
            let since = extract_id(&state, query.since_id, "0").await;
            queries::timeline::get_local_since(&state, &since, limit).await
        }
    } else {
        if let Some(max_id) = query.max_id {
            let until = extract_id(&state, Some(max_id), "9999").await;
            queries::timeline::get_federated(&state, &until, limit).await
        } else {
            let since = extract_id(&state, query.since_id, "0").await;
            queries::timeline::get_federated_since(&state, &since, limit).await
        }
    };

    let notes_json = timeline_json(&state, notes).await;

    Json(notes_json)
}

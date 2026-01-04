use crate::api::auth::OAuthUser;
use crate::back::init::AppState;
use crate::back::queries;

use axum::{
    Json,
    extract::{Query, State},
};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
pub struct TimelineQuery {
    pub limit: Option<i64>,
    pub since_id: Option<i64>,
    pub local: Option<bool>,
}

pub async fn extract_limit_and_since_id(state: &AppState, query: &TimelineQuery) -> (i64, String) {
    // Extract limit
    let limit = query.limit.unwrap_or(20);
    let limit = if limit > 40 { 40 } else { limit };

    // Extract since_id to since
    let since = if let Some(since_id) = query.since_id {
        let since_note = queries::note::get_by_id(&state, since_id).await;
        if let Some(since_note) = since_note {
            since_note.created_at
        } else {
            "0".to_string()
        }
    } else {
        "0".to_string()
    };

    (limit, since)
}

pub async fn timeline_json(notes: Vec<queries::note::NoteWithAuthorRecord>) -> Value {
    let notes_json: Value = notes
        .into_iter()
        .map(|note| {
            let attachments = if let Some(attachments) = note.attachments {
                if attachments.is_empty() {
                    vec![]
                } else {
                    attachments
                        .split("\n")
                        .map(|url| {
                            json!({
                                "type": "image",
                                "url": url,
                            })
                        })
                        .collect()
                }
            } else {
                vec![]
            };
            json!({
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
            })
        })
        .collect();

    notes_json
}

pub async fn get_home(
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
    user: OAuthUser,
) -> Json<Value> {
    let (limit, since) = extract_limit_and_since_id(&state, &query).await;

    let notes = queries::timeline::get_home_since(&state, user.id, &since, limit).await;

    let notes_json = timeline_json(notes).await;

    Json(notes_json)
}

pub async fn get_public(
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
) -> Json<Value> {
    let (limit, since) = extract_limit_and_since_id(&state, &query).await;

    let notes = if query.local.unwrap_or(false) {
        queries::timeline::get_local_since(&state, &since, limit).await
    } else {
        queries::timeline::get_federated_since(&state, &since, limit).await
    };

    let notes_json = timeline_json(notes).await;

    Json(notes_json)
}

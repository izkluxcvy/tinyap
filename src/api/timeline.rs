use crate::api::accounts::account_json;
use crate::api::auth::OAuthUser;
use crate::api::statuses::status_json;
use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    extract::{Query, State},
};
use serde_json::Value;

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

pub fn timeline_json(state: &AppState, notes: Vec<queries::note::NoteWithAuthorRecord>) -> Value {
    let notes_json: Value = notes
        .into_iter()
        .map(|note| {
            let attachments = utils::attachments_to_value(state, &note.attachments);

            let author_json = account_json(
                state,
                &note.username,
                &note.display_name,
                &note.created_at,
                "",
                0,
                0,
                0,
                &note.created_at,
            );
            let boosted_author_json = if let Some(boosted_username) = &note.boosted_username {
                Some(account_json(
                    state,
                    boosted_username,
                    boosted_username,
                    "9999-01-01T00:00:00Z",
                    "",
                    0,
                    0,
                    0,
                    "9999-01-01T00:00:00Z",
                ))
            } else {
                None
            };

            status_json(
                state,
                note.id,
                &note.username,
                note.boosted_id,
                note.boosted_created_at,
                boosted_author_json,
                &note.content,
                &author_json,
                &note.created_at,
                &attachments,
                note.like_count,
                note.boost_count,
                false,
                false,
                note.parent_id,
                note.parent_author_username,
            )
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
        let (until_date, until_id) = utils::extract_until_id(&state, Some(max_id)).await;
        queries::timeline::get_home(&state, user.id, &until_date, until_id, limit).await
    } else {
        let (since_date, since_id) = utils::extract_since_id(&state, query.since_id).await;
        queries::timeline::get_home_since(&state, user.id, &since_date, since_id, limit).await
    };

    let notes_json = timeline_json(&state, notes);

    Json(notes_json)
}

pub async fn get_public(
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
) -> Json<Value> {
    let limit = extract_limit(query.limit).await;

    let notes = if query.local.unwrap_or(false) {
        if let Some(max_id) = query.max_id {
            let (until_date, until_id) = utils::extract_until_id(&state, Some(max_id)).await;
            queries::timeline::get_local(&state, &until_date, until_id, limit).await
        } else {
            let (since_date, since_id) = utils::extract_since_id(&state, query.since_id).await;
            queries::timeline::get_local_since(&state, &since_date, since_id, limit).await
        }
    } else {
        if let Some(max_id) = query.max_id {
            let (until_date, until_id) = utils::extract_until_id(&state, Some(max_id)).await;
            queries::timeline::get_federated(&state, &until_date, until_id, limit).await
        } else {
            let (since_date, since_id) = utils::extract_since_id(&state, query.since_id).await;
            queries::timeline::get_federated_since(&state, &since_date, since_id, limit).await
        }
    };

    let notes_json = timeline_json(&state, notes);

    Json(notes_json)
}

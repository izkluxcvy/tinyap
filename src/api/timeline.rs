use crate::api::accounts::account_json;
use crate::api::auth::OAuthUser;
use crate::api::statuses::status_json;
use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use serde_json::Value;

fn build_link_header(
    domain: &str,
    path: &str,
    extra_params: &str,
    oldest_id: i64,
    newest_id: i64,
) -> String {
    let base = format!("https://{}{}", domain, path);
    let extra = if extra_params.is_empty() {
        String::new()
    } else {
        format!("&{}", extra_params)
    };
    format!(
        "<{}?max_id={}{}>; rel=\"next\", <{}?since_id={}{}>; rel=\"prev\"",
        base, oldest_id, extra, base, newest_id, extra,
    )
}

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
            let boosted_author_json = note.boosted_username.as_ref().map(|boosted_username| {
                account_json(
                    state,
                    boosted_username,
                    boosted_username,
                    "9999-01-01T00:00:00Z",
                    "",
                    0,
                    0,
                    0,
                    "9999-01-01T00:00:00Z",
                )
            });

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
) -> impl IntoResponse {
    let limit = extract_limit(query.limit).await;

    let notes = if let Some(max_id) = query.max_id {
        let (until_date, until_id) = utils::extract_until_id(&state, Some(max_id)).await;
        queries::timeline::get_home(&state, user.id, &until_date, until_id, limit).await
    } else {
        let (since_date, since_id) = utils::extract_since_id(&state, query.since_id).await;
        queries::timeline::get_home_since(&state, user.id, &since_date, since_id, limit).await
    };

    let mut headers = HeaderMap::new();
    if let (Some(first), Some(last)) = (notes.first(), notes.last()) {
        let link = build_link_header(
            &state.domain,
            "/api/v1/timelines/home",
            "",
            last.id,
            first.id,
        );
        headers.insert("Link", link.parse().unwrap());
    }

    let notes_json = timeline_json(&state, notes);
    (headers, Json(notes_json))
}

pub async fn get_public(
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
) -> impl IntoResponse {
    let limit = extract_limit(query.limit).await;
    let is_local = query.local.unwrap_or(false);

    let notes = if is_local {
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

    let extra_params = if is_local { "local=true" } else { "" };
    let mut headers = HeaderMap::new();
    if let (Some(first), Some(last)) = (notes.first(), notes.last()) {
        let link = build_link_header(
            &state.domain,
            "/api/v1/timelines/public",
            extra_params,
            last.id,
            first.id,
        );
        headers.insert("Link", link.parse().unwrap());
    }

    let notes_json = timeline_json(&state, notes);
    (headers, Json(notes_json))
}

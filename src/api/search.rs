use crate::api::accounts::account_json;
use crate::api::auth::OAuthUser;
use crate::api::timeline::{extract_limit, timeline_json};
use crate::back::init::AppState;
use crate::back::search;
use crate::back::utils;

use axum::{
    Json,
    extract::{Query, State},
};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    q: String,
    limit: Option<i64>,
    max_id: Option<i64>,
}

pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
    _user: OAuthUser,
) -> Json<Value> {
    let limit = extract_limit(query.limit).await;
    let (until_date, until_id) = utils::extract_until_id(&state, query.max_id).await;

    let Ok(result) = search::search(&state, &query.q, &until_date, until_id, limit).await else {
        return Json(json!({
            "accounts": [],
            "statuses": [],
            "hashtags": []
        }));
    };

    let accounts_json: Vec<Value> = result
        .user
        .iter()
        .map(|user| {
            account_json(
                &state,
                &user.username,
                &user.display_name,
                &user.created_at,
                &user.bio,
                user.follower_count,
                user.following_count,
                user.note_count,
                &user.updated_at,
            )
        })
        .collect();

    let mut notes = result.notes;
    if let Some(note) = result.note {
        notes.insert(0, note);
    }
    let statuses_json = timeline_json(&state, notes);

    Json(json!({
        "accounts": accounts_json,
        "statuses": statuses_json,
        "hashtags": []
    }))
}

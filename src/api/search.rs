use crate::api::accounts::account_json;
use crate::api::auth::OAuthUser;
use crate::api::statuses::status_json;
use crate::back::init::AppState;
use crate::back::queries;
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
}

pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
    _user: OAuthUser,
) -> Json<Value> {
    let Ok(path) = search::search(&state, &query.q).await else {
        return Json(json!({
            "error": "Not found"
        }));
    };

    let parts: Vec<&str> = path.split("/").collect();
    if query.q.starts_with("@") {
        let username = parts[1].trim_start_matches("@");
        let Some(user) = queries::user::get_by_username(&state, username).await else {
            return Json(json!({"error": "User not found"}));
        };

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

        Json(json!({
            "accounts": [account_json],
            "statuses": [],
            "hashtags": []
        }))
    } else {
        let note_id = parts[2].parse::<i64>().unwrap();
        let Some(note) = queries::note::get_with_author_by_id(&state, note_id).await else {
            return Json(json!({"error": "Note not found"}));
        };

        let attachments = utils::attachments_to_value(&state, &note.attachments);

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
            false,
            false,
            note.parent_id,
            note.parent_author_username,
        );

        Json(json!({
            "accounts": [],
            "statuses": [status_json],
            "hashtags": []
        }))
    }
}

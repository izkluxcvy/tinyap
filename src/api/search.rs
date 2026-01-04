use crate::api::auth::OAuthUser;
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

        return Json(json!({
            "accounts": [{
                "id": user.id,
                "username": &user.username,
                "acct": &user.username,
                "display_name": &user.display_name,
                "note": &user.bio,
                "created_at": &user.created_at,
                "followers_count": user.follower_count,
                "following_count": user.following_count,
                "statuses_count": user.note_count,
            }]
        }));
    } else {
        let note_id = parts[2].parse::<i64>().unwrap();
        let Some(note) = queries::note::get_with_author_by_id(&state, note_id).await else {
            return Json(json!({"error": "Note not found"}));
        };

        let attachments = utils::attachments_to_value(&state, &note.attachments);

        return Json(json!({
            "statuses": [{
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
            }]
        }));
    }
}

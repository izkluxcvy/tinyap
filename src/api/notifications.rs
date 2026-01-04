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
pub struct NotificationsQuery {
    pub since_id: Option<String>,
    pub limit: Option<i64>,
}

pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<NotificationsQuery>,
    user: OAuthUser,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(40);
    let limit = if limit > 80 { 80 } else { limit };
    let since = query.since_id.unwrap_or("0".to_string());

    let notifications = queries::notification::get_with_note(&state, user.id, &since, limit).await;

    let notifications_json: Value = notifications
        .into_iter()
        .map(|notif| {
            let attachments = utils::attachments_to_value(&state, &notif.attachments);
            let event_type = match notif.event_type {
                1 => "follow",
                2 => "mention",
                3 => "favourite",
                4 => "reblog",
                _ => "unknown",
            };

            json!({
                "id": &notif.created_at,
                "type": event_type,
                "created_at": &notif.created_at,
                "account": {
                    "id": notif.sender_id,
                    "username": &notif.username,
                    "acct": &notif.username,
                    "display_name": &notif.display_name,
                },
                "status": {
                    "id": notif.note_id,
                    "created_at": &notif.note_created_at,
                    "in_reply_to_id": notif.parent_id,
                    "visibility": "public",
                    "reblogs_count": notif.boost_count,
                    "favourites_count": notif.like_count,
                    "content": notif.content,
                    "attachments": attachments,
                }
            })
        })
        .collect();

    Json(json!(notifications_json))
}

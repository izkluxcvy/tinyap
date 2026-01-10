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

    // Get user
    let user = queries::user::get_by_id(&state, user.id).await;
    let user_json = account_json(
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

    // Get notifications
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

            let account_json = account_json(
                &state,
                &notif.username,
                &notif.display_name,
                &notif.created_at,
                "",
                0,
                0,
                0,
                &notif.created_at,
            );
            let status_json = if let Some(note_id) = notif.note_id {
                Some(status_json(
                    &state,
                    note_id,
                    "",
                    None,
                    None,
                    None,
                    notif.content.as_ref().unwrap_or(&"".to_string()),
                    &user_json,
                    notif.note_created_at.as_ref().unwrap_or(&"".to_string()),
                    &attachments,
                    notif.like_count.unwrap_or(0),
                    notif.boost_count.unwrap_or(0),
                    false,
                    false,
                    notif.parent_id,
                    None,
                ))
            } else {
                None
            };

            json!({
                "id": &notif.created_at,
                "type": event_type,
                "created_at": &notif.created_at,
                "account": &account_json,
                "status": &status_json,
            })
        })
        .collect();

    Json(json!(notifications_json))
}

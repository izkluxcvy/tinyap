use crate::api::auth::OAuthUser;
use crate::back::init::AppState;
use crate::back::queries;
use crate::back::user;
use crate::back::utils;

use axum::{
    Json,
    extract::{Multipart, State},
};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
pub struct UpdateCredentialsRequest {
    pub display_name: Option<String>,
    pub note: Option<String>,
}

pub async fn patch(
    State(state): State<AppState>,
    user: OAuthUser,
    mut multipart: Multipart,
) -> Json<Value> {
    // Parse multipart
    let mut req = UpdateCredentialsRequest {
        display_name: None,
        note: None,
    };
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();
        let value = field.text().await.unwrap_or("".to_string());
        match name.as_str() {
            "display_name" => req.display_name = Some(value),
            "note" => req.note = Some(value),
            _ => {}
        }
    }

    let user = queries::user::get_by_id(&state, user.id).await;

    let display_name = req.display_name.unwrap_or(user.display_name);
    let bio = req.note.unwrap_or(user.bio);
    let bio = utils::parse_content(&state, &bio);

    user::update_profile(&state, user.id, &display_name, &bio).await;

    Json(json!({
        "id": &user.username,
        "username": &user.username,
        "display_name": &display_name,
        "created_at": &user.created_at,
        "note": &bio,
        "url": &format!("https://{}/@{}", state.domain, &user.username),
        "followers_count": user.follower_count,
        "following_count": user.following_count,
        "statuses_count": user.note_count,
        "last_status_at": user.updated_at,
        "source": {
            "privacy": "public",
            "sensitive": false,
        }
    }))
}

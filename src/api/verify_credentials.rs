use crate::api::auth::OAuthUser;
use crate::back::init::AppState;
use crate::back::queries;

use axum::{Json, extract::State};
use serde_json::{Value, json};

pub async fn get(State(state): State<AppState>, user: OAuthUser) -> Json<Value> {
    let user = queries::user::get_by_id(&state, user.id).await;

    Json(json!({
        "id": &user.username,
        "username": &user.username,
        "display_name": &user.display_name,
        "created_at": &user.created_at,
        "note": &user.bio,
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

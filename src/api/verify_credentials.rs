use crate::api::accounts::account_json;
use crate::api::auth::OAuthUser;
use crate::back::init::AppState;
use crate::back::queries;

use axum::{Json, extract::State};
use serde_json::Value;

pub async fn get(State(state): State<AppState>, user: OAuthUser) -> Json<Value> {
    let user = queries::user::get_by_id(&state, user.id).await;

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

    Json(account_json)
}

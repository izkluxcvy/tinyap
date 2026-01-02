use crate::back::init::AppState;
use crate::back::queries;
use crate::web::auth::MaybeAuthUser;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};

#[derive(serde::Deserialize)]
pub struct PageQuery {
    until: Option<String>,
}

pub async fn get(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<PageQuery>,
    auth_user: MaybeAuthUser,
) -> impl IntoResponse {
    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return (StatusCode::NOT_FOUND, "User not found").into_response();
    };

    // Get notes by user
    let until = query.until.unwrap_or("9999-01-01-T00:00:00Z".to_string());
    let notes =
        queries::timeline::get_user(&state, user.id, &until, state.web_config.max_timeline_items)
            .await;
    let until_next = if let Some(last_note) = notes.last() {
        &last_note.created_at
    } else {
        &until
    };

    // Check if auth user follows this user
    // 0: not following, 1: following, 2: pending, 3: self
    let following_status: u8;
    if let Some(auth_user_id) = auth_user.id {
        let follow = queries::follow::get(&state, auth_user_id, user.id).await;
        if let Some(follow) = follow {
            if follow.pending == 1 {
                following_status = 2;
            } else {
                following_status = 1;
            }
        } else {
            if auth_user_id == user.id {
                following_status = 3;
            } else {
                following_status = 0;
            }
        }
    } else {
        following_status = 0;
    }

    // Rendering
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("timezone", &state.web_config.timezone);
    context.insert("domain", &state.domain);
    context.insert("user", &user);
    context.insert("following_status", &following_status);
    context.insert("notes", &notes);
    context.insert("until_next", until_next);
    context.insert("max_notes", &state.web_config.max_timeline_items);
    let rendered = state.tera.render("user.html", &context).unwrap();
    Html(rendered).into_response()
}

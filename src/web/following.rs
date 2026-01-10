use crate::back::init::AppState;
use crate::back::queries;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};

#[derive(serde::Deserialize)]
pub struct FollowingQuery {
    pub max: Option<String>,
}

pub async fn get_following(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<FollowingQuery>,
) -> impl IntoResponse {
    // extract max
    let max_username = query.max.unwrap_or("".to_string());

    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return (StatusCode::NOT_FOUND, "User not found").into_response();
    };

    // Get following
    let following = queries::follow::get_following(
        &state,
        user.id,
        &max_username,
        state.web_config.max_timeline_items,
    )
    .await;

    let max_next = if let Some(last) = following.last() {
        &last.username
    } else {
        ""
    };

    // Render
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("title", "Following");
    context.insert("username", &username);
    context.insert("users", &following);
    context.insert("max_next", max_next);
    context.insert("max_users", &state.web_config.max_timeline_items);
    let rendered = state.tera.render("following.html", &context).unwrap();

    Html(rendered).into_response()
}

pub async fn get_followers(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<FollowingQuery>,
) -> impl IntoResponse {
    // extract max
    let max_username = query.max.unwrap_or("".to_string());

    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return (StatusCode::NOT_FOUND, "User not found").into_response();
    };

    // Get followers
    let followers = queries::follow::get_followers(
        &state,
        user.id,
        &max_username,
        state.web_config.max_timeline_items,
    )
    .await;

    let max_next = if let Some(last) = followers.last() {
        &last.username
    } else {
        ""
    };

    // Render
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("title", "Followers");
    context.insert("username", &username);
    context.insert("users", &followers);
    context.insert("max_next", max_next);
    context.insert("max_users", &state.web_config.max_timeline_items);
    let rendered = state.tera.render("following.html", &context).unwrap();

    Html(rendered).into_response()
}

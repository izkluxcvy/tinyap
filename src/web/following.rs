use crate::back::init::AppState;
use crate::back::queries;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};

pub async fn get_following(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return (StatusCode::NOT_FOUND, "User not found").into_response();
    };

    // Get following
    let following = queries::follow::get_following(&state, user.id).await;

    // Render
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("title", "Following");
    context.insert("username", &username);
    context.insert("users", &following);
    let rendered = state.tera.render("following.html", &context).unwrap();

    Html(rendered).into_response()
}

pub async fn get_followers(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    // Get user
    let Some(user) = queries::user::get_by_username(&state, &username).await else {
        return (StatusCode::NOT_FOUND, "User not found").into_response();
    };

    // Get followers
    let followers = queries::follow::get_followers(&state, user.id).await;

    // Render
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("title", "Followers");
    context.insert("username", &username);
    context.insert("users", &followers);
    let rendered = state.tera.render("following.html", &context).unwrap();

    Html(rendered).into_response()
}

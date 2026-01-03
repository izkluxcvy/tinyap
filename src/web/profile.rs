use crate::back::init::AppState;
use crate::back::queries;
use crate::back::user;
use crate::back::utils;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect},
};

pub async fn get(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    let user = queries::user::get_by_id(&state, user.id).await;
    let bio = utils::strip_content(&state, &user.bio);

    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("display_name", &user.display_name);
    context.insert("bio", &bio);
    let rendered = state.tera.render("profile.html", &context).unwrap();

    Html(rendered)
}

#[derive(serde::Deserialize)]
pub struct ProfileForm {
    pub display_name: String,
    pub bio: String,
}

pub async fn post_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<ProfileForm>,
) -> impl IntoResponse {
    user::update_profile(&state, user.id, &form.display_name, &form.bio).await;

    Redirect::to("/home").into_response()
}

#[derive(serde::Deserialize)]
pub struct ChangePasswordForm {
    pub current_password: String,
    pub new_password: String,
    pub confirmation: String,
}

pub async fn post_password(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<ChangePasswordForm>,
) -> impl IntoResponse {
    // Check new password and confirmation match
    if form.new_password != form.confirmation {
        return "passwords do not match.".into_response();
    }

    // Verify current password
    let user = queries::user::get_by_id(&state, user.id).await;
    let verify = user::verify_password(&state, &user.username, &form.current_password).await;
    if verify.is_err() {
        return "current password is incorrect.".into_response();
    }

    // Update password
    user::update_password(&state, user.id, &form.new_password).await;

    Redirect::to("/home").into_response()
}

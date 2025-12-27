use crate::back::init::AppState;
use crate::back::queries;
use crate::back::user;
use crate::back::utils;

use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

pub async fn get(State(state): State<AppState>) -> Html<String> {
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    let rendered = state.tera.render("login.html", &context).unwrap();
    Html(rendered)
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn post(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let Ok(user_id) = user::verify_password(&state, &form.username, &form.password).await else {
        return "Incorrect username or password.".into_response();
    };

    let session_id = utils::gen_secure_token();
    let expires_at = utils::date_plus_days(state.web_config.session_ttl_days);
    queries::session::create(&state, &session_id, &user_id, &expires_at).await;
    let date_now = utils::date_now();
    queries::session::delete_old(
        &state,
        &user_id,
        &state.web_config.max_sessions_per_user,
        &date_now,
    )
    .await;

    let cookie = Cookie::build(("session_id", session_id))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(true)
        .max_age(time::Duration::days(state.web_config.session_ttl_days));

    (jar.add(cookie), Redirect::to("/home")).into_response()
}

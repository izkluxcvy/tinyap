use crate::back::init::AppState;
use crate::back::user;

use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect},
};

pub async fn get(State(state): State<AppState>) -> Html<String> {
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("allow_signup", &state.web_config.allow_signup);
    let rendered = state.tera.render("signup.html", &context).unwrap();
    Html(rendered)
}

#[derive(serde::Deserialize)]
pub struct SignupForm {
    pub username: String,
    pub password: String,
    pub confirmation: String,
}

pub async fn post(
    State(state): State<AppState>,
    Form(form): Form<SignupForm>,
) -> impl IntoResponse {
    if !state.web_config.allow_signup {
        return "Signup is not allowed.".into_response();
    }

    if form.password != form.confirmation {
        return "Passwords do not match.".into_response();
    }

    let res = user::add(&state, &form.username, &form.password).await;
    match res {
        Ok(_) => Redirect::to("/login").into_response(),
        Err(e) => e.into_response(),
    }
}

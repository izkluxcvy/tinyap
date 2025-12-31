use crate::back::init::AppState;
use crate::back::search;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect},
};

pub async fn get(State(state): State<AppState>, _user: AuthUser) -> Html<String> {
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    let rendered = state.tera.render("search.html", &context).unwrap();

    Html(rendered)
}

#[derive(serde::Deserialize)]
pub struct SearchForm {
    pub q: String,
}

pub async fn post(
    State(state): State<AppState>,
    _user: AuthUser,
    Form(form): Form<SearchForm>,
) -> impl IntoResponse {
    match search::search(&state, &form.q).await {
        Ok(link) => Redirect::to(&link).into_response(),
        Err(e) => e.into_response(),
    }
}

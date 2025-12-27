use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect},
};

pub async fn get(State(state): State<AppState>, _user: AuthUser) -> Html<String> {
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    let rendered = state.tera.render("new.html", &context).unwrap();

    Html(rendered)
}

#[derive(serde::Deserialize)]
pub struct NewNoteForm {
    pub content: String,
}

pub async fn post(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<NewNoteForm>,
) -> impl IntoResponse {
    let user = queries::user::get_by_id(&state, user.id).await;

    let id = utils::gen_unique_id();
    let ap_id = utils::local_note_apid(&state.domain, id);
    let content = utils::parse_content(&form.content);
    let created_at = utils::date_now();

    if content.is_empty() {
        return "Content cannot be empty".into_response();
    }

    queries::note::create(&state, &id, &ap_id, &user.id, &content, &created_at, &1).await;

    Redirect::to("/home").into_response()
}

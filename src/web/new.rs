use crate::back::init::AppState;
use crate::back::note;
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
    let res = note::add(
        &state,
        user.id,
        None,
        None,
        &form.content,
        None,
        None,
        None,
        1, // is_public
    )
    .await;

    match res {
        Ok(_) => Redirect::to("/home").into_response(),
        Err(e) => {
            println!("Error adding note: {}", e);
            "Something went wrong".into_response()
        }
    }
}

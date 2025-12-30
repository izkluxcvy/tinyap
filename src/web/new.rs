use crate::back::init::AppState;
use crate::back::note;
use crate::back::queries;
use crate::back::utils;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Form, Query, State},
    response::{Html, IntoResponse, Redirect},
};

#[derive(serde::Deserialize)]
pub struct NewNoteQuery {
    pub parent_id: Option<i64>,
}

pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<NewNoteQuery>,
    _user: AuthUser,
) -> Html<String> {
    let parent = if let Some(parent_id) = query.parent_id {
        queries::note::get_with_author_by_id(&state, parent_id).await
    } else {
        None
    };

    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("parent_id", &query.parent_id);
    context.insert("parent", &parent);
    context.insert("timezone", &state.web_config.timezone);
    let rendered = state.tera.render("new.html", &context).unwrap();

    Html(rendered)
}

#[derive(serde::Deserialize)]
pub struct NewNoteForm {
    pub content: String,
    pub parent_id: Option<i64>,
}

pub async fn post(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<NewNoteForm>,
) -> impl IntoResponse {
    let id = utils::gen_unique_id();
    let ap_url = utils::local_note_ap_url(&state.domain, id);
    let created_at = utils::date_now();

    // in_reply_to handling
    let parent_author_username = if let Some(parent_id) = form.parent_id {
        let Some(parent) = queries::note::get_by_id(&state, parent_id).await else {
            return "Parent note not found".into_response();
        };
        let parent_author = queries::user::get_by_id(&state, parent.author_id).await;
        Some(parent_author.username)
    } else {
        None
    };

    // Create note
    let res = note::add(
        &state,
        id,
        &ap_url,
        user.id,
        &form.content,
        None,
        form.parent_id,
        parent_author_username,
        &created_at,
        1, // is_public
    )
    .await;

    if let Err(e) = res {
        println!("Error creating note: {}", e);
        return "Something went wrong".into_response();
    }

    // Deliver to followers and parent
    note::deliver_create(&state, id, user.id).await;

    Redirect::to("/home").into_response()
}

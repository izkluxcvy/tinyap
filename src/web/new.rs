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
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("parent_id", &query.parent_id);
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
    let created_at = utils::date_now();

    // in_reply_to handling
    let parent_inbox: Option<String>;
    let parent_author_username = if let Some(parent_id) = form.parent_id {
        let Some(parent) = queries::note::get_by_id(&state, parent_id).await else {
            return "Parent note not found".into_response();
        };
        let parent_author = queries::user::get_by_id(&state, parent.author_id).await;
        parent_inbox = Some(parent_author.inbox_url.clone());
        Some(parent_author.username)
    } else {
        parent_inbox = None;
        None
    };

    // Create note
    let res = note::add(
        &state,
        id,
        user.id,
        None,
        None,
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
    let create_activity = note::create_activity(&state, id, user.id).await;
    let json_body = create_activity.to_string();
    utils::deliver_to_followers(&state, user.id, parent_inbox, &json_body).await;

    Redirect::to("/home").into_response()
}

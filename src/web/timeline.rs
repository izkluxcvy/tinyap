use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils::extract_until_id;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Query, State},
    response::Html,
};

#[derive(serde::Deserialize)]
pub struct PageQuery {
    pub until: Option<i64>,
}

pub async fn get_home(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<PageQuery>,
) -> Html<String> {
    // Get user
    let user = queries::user::get_by_id(&state, user.id).await;

    // Get notes
    let (until_date, until_id) = extract_until_id(&state, query.until).await;
    let notes = queries::timeline::get_home(
        &state,
        user.id,
        &until_date,
        until_id,
        state.web_config.max_timeline_items,
    )
    .await;
    let until_next = if let Some(last_note) = notes.last() {
        last_note.id
    } else {
        until_id
    };

    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("title", "Home");
    context.insert("display_name", &user.display_name);
    context.insert("username", &user.username);
    context.insert("timezone", &state.web_config.timezone);
    context.insert("notes", &notes);
    context.insert("until_next", &until_next);
    context.insert("max_notes", &state.web_config.max_timeline_items);
    let rendered = state.tera.render("timeline.html", &context).unwrap();

    Html(rendered)
}

pub async fn get_local(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> Html<String> {
    let (until_date, until_id) = extract_until_id(&state, query.until).await;
    let notes = queries::timeline::get_local(
        &state,
        &until_date,
        until_id,
        state.web_config.max_timeline_items,
    )
    .await;
    let until_next = if let Some(last_note) = notes.last() {
        last_note.id
    } else {
        until_id
    };

    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("title", "Local Timeline");
    context.insert("timezone", &state.web_config.timezone);
    context.insert("notes", &notes);
    context.insert("until_next", &until_next);
    context.insert("max_notes", &state.web_config.max_timeline_items);
    let rendered = state.tera.render("timeline.html", &context).unwrap();

    Html(rendered)
}

pub async fn get_federated(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> Html<String> {
    let (until_date, until_id) = extract_until_id(&state, query.until).await;
    let notes = queries::timeline::get_federated(
        &state,
        &until_date,
        until_id,
        state.web_config.max_timeline_items,
    )
    .await;
    let until_next = if let Some(last_note) = notes.last() {
        last_note.id
    } else {
        until_id
    };

    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("title", "Federated Timeline");
    context.insert("timezone", &state.web_config.timezone);
    context.insert("notes", &notes);
    context.insert("until_next", &until_next);
    context.insert("max_notes", &state.web_config.max_timeline_items);
    let rendered = state.tera.render("timeline.html", &context).unwrap();

    Html(rendered)
}

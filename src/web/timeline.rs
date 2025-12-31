use crate::back::init::AppState;
use crate::back::queries;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Query, State},
    response::Html,
};

#[derive(serde::Deserialize)]
pub struct PageQuery {
    pub until: Option<String>,
}

pub async fn get_home(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<PageQuery>,
) -> Html<String> {
    // Get user
    let user = queries::user::get_by_id(&state, user.id).await;

    // Get notes
    let until = query.until.unwrap_or("9999-01-01-T00:00:00Z".to_string());
    let notes =
        queries::timeline::get_home(&state, user.id, &until, state.web_config.max_timeline_items)
            .await;
    let until_next = if let Some(last_note) = notes.last() {
        &last_note.created_at
    } else {
        &until
    };

    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("title", "Home");
    context.insert("display_name", &user.display_name);
    context.insert("username", &user.username);
    context.insert("timezone", &state.web_config.timezone);
    context.insert("notes", &notes);
    context.insert("until_next", until_next);
    context.insert("max_notes", &state.web_config.max_timeline_items);
    let rendered = state.tera.render("timeline.html", &context).unwrap();

    Html(rendered)
}

pub async fn get_local(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> Html<String> {
    let until = query.until.unwrap_or("9999-01-01-T00:00:00Z".to_string());
    let notes =
        queries::timeline::get_local(&state, &until, state.web_config.max_timeline_items).await;
    let until_next = if let Some(last_note) = notes.last() {
        &last_note.created_at
    } else {
        &until
    };

    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("title", "Local Timeline");
    context.insert("timezone", &state.web_config.timezone);
    context.insert("notes", &notes);
    context.insert("until_next", until_next);
    context.insert("max_notes", &state.web_config.max_timeline_items);
    let rendered = state.tera.render("timeline.html", &context).unwrap();

    Html(rendered)
}

pub async fn get_federated(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> Html<String> {
    let until = query.until.unwrap_or("9999-01-01-T00:00:00Z".to_string());
    let notes =
        queries::timeline::get_federated(&state, &until, state.web_config.max_timeline_items).await;
    let until_next = if let Some(last_note) = notes.last() {
        &last_note.created_at
    } else {
        &until
    };

    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("title", "Federated Timeline");
    context.insert("timezone", &state.web_config.timezone);
    context.insert("notes", &notes);
    context.insert("until_next", until_next);
    context.insert("max_notes", &state.web_config.max_timeline_items);
    let rendered = state.tera.render("timeline.html", &context).unwrap();

    Html(rendered)
}

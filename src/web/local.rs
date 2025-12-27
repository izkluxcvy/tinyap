use crate::back::init::AppState;
use crate::back::queries;

use axum::{extract::State, response::Html};

pub async fn get(State(state): State<AppState>) -> Html<String> {
    let notes = queries::timeline::get_local(&state, state.web_config.max_timeline_items).await;

    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("title", "Local Timeline");
    context.insert("timezone", &state.web_config.timezone);
    context.insert("notes", &notes);
    let rendered = state.tera.render("timeline.html", &context).unwrap();

    Html(rendered)
}

use crate::back::init::AppState;

use axum::{extract::State, response::Html};

pub async fn get(State(state): State<AppState>) -> Html<String> {
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("instance_description", &state.metadata.instance_description);
    let rendered = state.tera.render("index.html", &context).unwrap();

    Html(rendered)
}

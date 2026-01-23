use crate::back::init::AppState;

use axum::{extract::State, response::Html};

pub async fn get(State(state): State<AppState>) -> Html<String> {
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("instance_description", &state.metadata.instance_description);
    context.insert("admin_username", &state.metadata.admin_username);
    context.insert("admin_email", &state.metadata.admin_email);
    let rendered = state.tera.render("index.html", &context).unwrap();

    Html(rendered)
}

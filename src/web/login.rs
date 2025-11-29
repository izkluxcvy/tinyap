use crate::state::AppState;

use axum::{extract::State, response::Html};

pub async fn page(State(state): State<AppState>) -> Html<String> {
    let rendered = state
        .tera
        .render("login.html", &tera::Context::new())
        .unwrap();
    Html(rendered)
}

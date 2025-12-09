use crate::state::AppState;

use axum::{extract::State, response::Html};

pub async fn page(State(state): State<AppState>) -> Html<String> {
    let mut context = tera::Context::new();
    context.insert("site_name", &state.site_name);
    let rendered = state.tera.render("logout.html", &context).unwrap();
    Html(rendered)
}

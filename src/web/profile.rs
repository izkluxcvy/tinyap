use crate::auth::AuthUser;
use crate::state::AppState;

use axum::{extract::State, response::Html};

pub async fn page(State(state): State<AppState>, user: AuthUser) -> Html<String> {
    let row = sqlx::query!(
        "SELECT username, display_name, bio, created_at
        FROM users WHERE id = ?",
        user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    let mut context = tera::Context::new();
    context.insert("username", &row.username);
    context.insert("display_name", &row.display_name);
    context.insert("bio", &row.bio);
    context.insert("created_at", &row.created_at);
    let rendered = state.tera.render("profile.html", &context).unwrap();
    Html(rendered)
}

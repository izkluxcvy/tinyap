use crate::state::AppState;

use axum::{
    extract::{Path, State},
    response::Html,
};

pub async fn following(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Html<String> {
    let rows = sqlx::query!(
        "SELECT users.username
        FROM follows
        JOIN users ON follows.object_actor = users.actor_id
        WHERE follows.user_id = (SELECT id FROM users WHERE username = ?)
        AND follows.pending = 0",
        username
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap();

    let follows: Vec<String> = rows.into_iter().map(|row| row.username).collect();

    let mut context = tera::Context::new();
    context.insert("site_name", &state.site_name);
    context.insert("title", &format!("@{}'s Following", username));
    context.insert("users", &follows);
    let rendered = state.tera.render("follows.html", &context).unwrap();

    Html(rendered)
}

pub async fn followers(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Html<String> {
    let rows = sqlx::query!(
        "SELECT users.username
        FROM followers
        JOIN users ON followers.actor = users.actor_id
        WHERE followers.user_id = (SELECT id FROM users WHERE username = ?)",
        username
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap();

    let followers: Vec<String> = rows.into_iter().map(|row| row.username).collect();

    let mut context = tera::Context::new();
    context.insert("site_name", &state.site_name);
    context.insert("title", &format!("@{}'s Followers", username));
    context.insert("users", &followers);
    let rendered = state.tera.render("follows.html", &context).unwrap();

    Html(rendered)
}

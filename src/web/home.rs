use crate::{auth::AuthUser, state::AppState};

use axum::{extract::State, response::Html};

pub async fn page(user: AuthUser, State(state): State<AppState>) -> Html<String> {
    let home_user = sqlx::query!(
        "SELECT id, display_name, username FROM users WHERE id = ?",
        user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .expect("Failed to fetch home user from database");

    // Fetch notes from users who home user follows
    let rows = sqlx::query!(
        "SELECT notes.uuid, notes.content, notes.in_reply_to, notes.created_at, users.display_name, users.username
        FROM notes
        JOIN users ON notes.user_id = users.id
        LEFT JOIN follows ON follows.object_actor = users.actor_id
        AND follows.user_id = $1
        WHERE follows.user_id = $1
        OR users.id = $1
        ORDER BY notes.created_at DESC
        LIMIT $2",
        home_user.id,
        state.config.max_timeline_notes
    )
    .fetch_all(&state.db_pool)
    .await
    .expect("Failed to fetch notes for home page");

    let notes: Vec<_> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "uuid": row.uuid,
                "display_name": row.display_name,
                "username": row.username,
                "content": row.content,
                "in_reply_to": row.in_reply_to,
                "created_at": row.created_at,
            })
        })
        .collect();

    let mut context = tera::Context::new();
    context.insert("display_name", &home_user.display_name);
    context.insert("username", &home_user.username);
    context.insert("notes", &notes);
    context.insert("timezone", &state.config.timezone);
    let rendered = state.tera.render("home.html", &context).unwrap();
    Html(rendered)
}

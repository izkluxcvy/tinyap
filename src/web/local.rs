use crate::state::AppState;

use axum::{extract::State, response::Html};

pub async fn page(State(state): State<AppState>) -> Html<String> {
    let rows = sqlx::query!(
        "SELECT notes.uuid, notes.content, notes.in_reply_to, notes.created_at, users.display_name, users.username
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE users.is_local = 1
        AND notes.boosted_username IS NULL
        ORDER BY notes.created_at DESC
        LIMIT ?",
        state.config.max_timeline_notes
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap();

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
    context.insert("title", "Local Timeline");
    context.insert("notes", &notes);
    context.insert("timezone", &state.config.timezone);
    let rendered = state.tera.render("timeline.html", &context).unwrap();
    Html(rendered)
}

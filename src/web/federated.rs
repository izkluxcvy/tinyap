use crate::state::AppState;

use axum::{
    extract::{Query, State},
    response::Html,
};

#[derive(serde::Deserialize)]
pub struct PageParam {
    p: Option<i64>,
}

pub async fn page(
    State(state): State<AppState>,
    Query(PageParam { p }): Query<PageParam>,
) -> Html<String> {
    let offset = (p.unwrap_or(1) - 1) * state.config.max_timeline_notes;
    let rows = sqlx::query!(
        "SELECT notes.uuid, notes.content, notes.in_reply_to, notes.reply_to_author, notes.created_at, users.display_name, users.username
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.boosted_username IS NULL
        AND notes.is_public = 1
        ORDER BY notes.created_at DESC
        LIMIT ?
        OFFSET ?",
        state.config.max_timeline_notes,
        offset
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
                "reply_to_author": row.reply_to_author,
                "created_at": row.created_at,
            })
        })
        .collect();

    let mut context = tera::Context::new();
    context.insert("site_name", &state.site_name);
    context.insert("title", "Federated Timeline");
    context.insert("notes", &notes);
    context.insert("timezone", &state.config.timezone);
    context.insert("page", &p.unwrap_or(1));
    context.insert("max_notes", &state.config.max_timeline_notes);
    let rendered = state.tera.render("timeline.html", &context).unwrap();
    Html(rendered)
}

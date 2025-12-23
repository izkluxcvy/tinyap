use crate::state::AppState;

use axum::{
    extract::{Query, State},
    response::Html,
};

#[derive(serde::Deserialize)]
pub struct PageParam {
    until: Option<String>,
}

pub async fn page(
    State(state): State<AppState>,
    Query(PageParam { until }): Query<PageParam>,
) -> Html<String> {
    let until = until.unwrap_or("9999-01-01T00:00:00Z".to_string());
    let rows = sqlx::query!(
        "SELECT notes.uuid, notes.content, notes.in_reply_to, notes.reply_to_author, notes.created_at, users.display_name, users.username
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.boosted_username IS NULL
        AND notes.is_public = 1
        AND notes.created_at < ?
        ORDER BY notes.created_at DESC
        LIMIT ?",
        until,
        state.config.max_timeline_notes,
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

    let until_next = if let Some(last_note) = notes.last() {
        &last_note["created_at"].as_str().unwrap().to_string()
    } else {
        &until
    };

    let mut context = tera::Context::new();
    context.insert("site_name", &state.site_name);
    context.insert("title", "Federated Timeline");
    context.insert("notes", &notes);
    context.insert("timezone", &state.config.timezone);
    context.insert("until_next", &until_next);
    context.insert("max_notes", &state.config.max_timeline_notes);
    let rendered = state.tera.render("timeline.html", &context).unwrap();
    Html(rendered)
}

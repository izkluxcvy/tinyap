use crate::{auth::AuthUser, state::AppState};

use axum::{
    extract::{Query, State},
    response::Html,
};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
pub struct ReplyParam {
    pub in_reply_to: Option<String>,
}

pub async fn page(
    _user: AuthUser,
    State(state): State<AppState>,
    Query(ReplyParam { in_reply_to }): Query<ReplyParam>,
) -> Html<String> {
    let mut reply_to_note: Option<Value> = None;
    if let Some(in_reply_to) = &in_reply_to {
        let row = sqlx::query!(
            "SELECT notes.uuid, notes.content, notes.in_reply_to, notes.reply_to_author, notes.created_at, users.display_name, users.username
            FROM notes
            JOIN users ON notes.user_id = users.id
            WHERE notes.ap_id = ?",
            in_reply_to
        )
        .fetch_optional(&state.db_pool)
        .await
        .unwrap();

        if let Some(row) = row {
            reply_to_note = Some(json!({
                "uuid": row.uuid,
                "display_name": row.display_name,
                "username": row.username,
                "content": row.content,
                "in_reply_to": row.in_reply_to,
                "reply_to_author": row.reply_to_author,
                "created_at": row.created_at,
            }))
        }
    }

    let mut context = tera::Context::new();
    context.insert("site_name", &state.site_name);
    context.insert("reply_to_note", &reply_to_note);
    context.insert("in_reply_to", &in_reply_to);
    context.insert("timezone", &state.config.timezone);
    let rendered = state.tera.render("new.html", &context).unwrap();
    Html(rendered)
}

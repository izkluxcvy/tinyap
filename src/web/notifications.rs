use crate::auth::AuthUser;
use crate::state::AppState;

use axum::{extract::State, response::Html};
use serde_json::json;

pub async fn page(State(state): State<AppState>, user: AuthUser) -> Html<String> {
    let rows = sqlx::query!(
        "SELECT notifs.username, notifs.type, notifs.actor, notifs.note_uuid, notifs.created_at
        FROM notifications AS notifs
        JOIN users ON notifs.username = users.username
        WHERE users.id = ?
        ORDER BY notifs.created_at DESC
        LIMIT ?",
        user.id,
        state.config.max_timeline_notes
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap();

    let mut notifs: Vec<_> = Vec::new();
    for row in rows {
        let actor_user = sqlx::query!("SELECT username FROM users WHERE actor_id = ?", row.actor)
            .fetch_optional(&state.db_pool)
            .await
            .unwrap();

        let actor_name = match actor_user {
            Some(u) => format!("@{}", u.username),
            None => row.actor,
        };

        let mut notif = json!({
            "username": row.username,
            "type": row.r#type,
            "actor": actor_name,
            "created_at": row.created_at,
        });
        if let Some(note_uuid) = row.note_uuid {
            notif["note_uuid"] = json!(note_uuid);
        }
        notifs.push(notif);
    }

    let mut context = tera::Context::new();
    context.insert("notifs", &notifs);
    let rendered = state.tera.render("notifications.html", &context).unwrap();
    Html(rendered)
}

use crate::back::init::AppState;

use sqlx::query_as;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct TimelineRecord {
    pub display_name: String,
    pub username: String,
    pub id: i64,
    pub content: String,
    pub attachments: Option<String>,
    pub created_at: String,
}

pub async fn get_user(state: &AppState, user_id: i64, limit: i64) -> Vec<TimelineRecord> {
    query_as(
        "SELECT u.display_name, u.username, n.id, n.content, n.attachments, n.created_at
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        WHERE u.id = ?
        AND n.is_public = 1
        ORDER BY n.created_at DESC
        LIMIT ?",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

pub async fn get_local(state: &AppState, limit: i64) -> Vec<TimelineRecord> {
    query_as(
        "SELECT u.display_name, u.username, n.id, n.content, n.attachments, n.created_at
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        WHERE u.is_local = 1
        AND n.is_public = 1
        ORDER BY n.created_at DESC
        LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

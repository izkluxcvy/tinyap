use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct NotificationRecord {
    pub display_name: String,
    pub username: String,
    pub event_type: i64,
    pub note_id: Option<i64>,
    pub created_at: String,
}

pub async fn get(state: &AppState, recipient_id: i64, limit: i64) -> Vec<NotificationRecord> {
    query_as(
        "SELECT u.display_name, u.username, n.event_type, n.note_id, n.created_at
        FROM notifications AS n
        JOIN users AS u ON n.sender_id = u.id
        WHERE n.recipient_id = ?
        ORDER BY n.created_at DESC
        LIMIT ?",
    )
    .bind(recipient_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

pub async fn create(
    state: &AppState,
    event_type: i64,
    sender_id: i64,
    recipient_id: i64,
    note_id: Option<i64>,
    created_at: &str,
) {
    query(
        "INSERT INTO notifications (event_type, sender_id, recipient_id, note_id, created_at)
        VALUES (?, ?, ?, ?, ?)",
    )
    .bind(event_type)
    .bind(sender_id)
    .bind(recipient_id)
    .bind(note_id)
    .bind(created_at)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

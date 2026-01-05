use crate::back::init::AppState;

use sqlx::{query, query_as};

#[cfg(feature = "web")]
#[derive(sqlx::FromRow, serde::Serialize)]
pub struct NotificationRecord {
    pub display_name: String,
    pub username: String,
    pub event_type: i64,
    pub note_id: Option<i64>,
    pub created_at: String,
}

#[cfg(feature = "web")]
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

#[cfg(feature = "api")]
#[derive(sqlx::FromRow)]
pub struct NotificationWithNoteRecord {
    pub display_name: String,
    pub username: String,
    pub event_type: i64,
    pub note_id: i64,
    pub content: String,
    pub attachments: Option<String>,
    pub note_created_at: String,
    pub parent_id: Option<i64>,
    pub like_count: i64,
    pub boost_count: i64,
    pub created_at: String,
}

#[cfg(feature = "api")]
pub async fn get_with_note(
    state: &AppState,
    recipient_id: i64,
    since: &str,
    limit: i64,
) -> Vec<NotificationWithNoteRecord> {
    query_as(
        "SELECT u.display_name, u.username, notif.event_type, notif.note_id, note.content, note.attachments, note.created_at AS note_created_at, note.parent_id, note.parent_author_username, note.like_count, note.boost_count, notif.created_at
        FROM notifications AS notif
        JOIN users AS u ON notif.sender_id = u.id
        JOIN notes AS note ON notif.note_id = note.id
        WHERE notif.recipient_id = ? AND notif.created_at > ?
        ORDER BY notif.created_at DESC
        LIMIT ?",
    )
    .bind(recipient_id)
    .bind(since)
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

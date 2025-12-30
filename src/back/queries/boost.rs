use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow)]
pub struct BoostRecord {
    pub _id: i64,
}

pub async fn get(state: &AppState, user_id: i64, note_id: i64) -> Option<BoostRecord> {
    query_as(
        "SELECT id AS _id FROM notes
        WHERE author_id = ?
        AND boosted_id = ?",
    )
    .bind(user_id)
    .bind(note_id)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap()
}

pub async fn create(
    state: &AppState,
    id: i64,
    ap_url: &str,
    author_id: i64,
    boosted_id: i64,
    boosted_username: &str,
    boosted_created_at: &str,
    content: &str,
    attachments: Option<String>,
    parent_id: Option<i64>,
    parent_author_username: Option<String>,
    created_at: &str,
) {
    query(
        "INSERT INTO notes (id, ap_url, author_id, boosted_id, boosted_username, boosted_created_at, content, attachments, parent_id, parent_author_username, created_at, is_public)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)",
    )
    .bind(id)
    .bind(ap_url)
    .bind(author_id)
    .bind(boosted_id)
    .bind(boosted_username)
    .bind(boosted_created_at)
    .bind(content)
    .bind(attachments)
    .bind(parent_id)
    .bind(parent_author_username)
    .bind(created_at)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn delete(state: &AppState, user_id: i64, note_id: i64) {
    query(
        "DELETE FROM notes
        WHERE author_id = ?
        AND boosted_id = ?",
    )
    .bind(user_id)
    .bind(note_id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

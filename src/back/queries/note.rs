use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow)]
pub struct NoteRecord {
    pub id: i64,
    pub ap_url: String,
    pub author_id: i64,
    pub content: String,
    pub attachments: Option<String>,
    pub created_at: String,
    pub is_public: i64,
}

pub async fn get_by_id(state: &AppState, id: &i64) -> Option<NoteRecord> {
    query_as("SELECT * FROM notes WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap()
}

pub async fn create(
    state: &AppState,
    id: &i64,
    ap_url: &str,
    author_id: &i64,
    content: &str,
    attachments: Option<&str>,
    created_at: &str,
    is_public: &i64,
) {
    query(
        "INSERT INTO notes (id, ap_url, author_id, content, attachments, created_at, is_public)
        VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(ap_url)
    .bind(author_id)
    .bind(content)
    .bind(attachments)
    .bind(created_at)
    .bind(is_public)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

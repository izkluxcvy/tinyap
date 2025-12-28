use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow)]
pub struct NoteRecord {
    pub id: i64,
    pub ap_url: String,
    pub author_id: i64,
    pub boosted_id: Option<i64>,
    pub boosted_username: Option<String>,
    pub content: String,
    pub attachments: Option<String>,
    pub in_reply_to: Option<String>,
    pub parent_author_username: Option<String>,
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

pub async fn get_by_ap_url(state: &AppState, ap_url: &str) -> Option<NoteRecord> {
    query_as("SELECT * FROM notes WHERE ap_url = ?")
        .bind(ap_url)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap()
}

pub async fn create(
    state: &AppState,
    id: i64,
    ap_url: &str,
    author_id: i64,
    boosted_id: Option<i64>,
    boosted_username: Option<&str>,
    content: &str,
    attachments: Option<&str>,
    in_reply_to: Option<&str>,
    parent_author_username: Option<&str>,
    created_at: &str,
    is_public: i64,
) {
    query(
        "INSERT INTO notes (id, ap_url, author_id, boosted_id, boosted_username, content, attachments, in_reply_to, parent_author_username, created_at, is_public)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(ap_url)
    .bind(author_id)
    .bind(boosted_id)
    .bind(boosted_username)
    .bind(content)
    .bind(attachments)
    .bind(in_reply_to)
    .bind(parent_author_username)
    .bind(created_at)
    .bind(is_public)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

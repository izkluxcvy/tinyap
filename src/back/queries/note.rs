use crate::back::init::AppState;

use sqlx::{query, query_as};

pub async fn create(
    state: &AppState,
    id: &i64,
    ap_id: &str,
    author_id: &i64,
    content: &str,
    created_at: &str,
    is_public: &i64,
) {
    query(
        "INSERT INTO notes (id, ap_id, author_id, content, created_at, is_public)
        VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(ap_id)
    .bind(author_id)
    .bind(content)
    .bind(created_at)
    .bind(is_public)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

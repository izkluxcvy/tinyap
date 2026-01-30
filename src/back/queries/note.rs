use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow)]
pub struct NoteRecord {
    pub id: i64,
    pub ap_url: String,
    pub author_id: i64,
    pub content: String,
    pub attachments: Option<String>,
    pub parent_id: Option<i64>,
    pub parent_author_username: Option<String>,
    pub created_at: String,
    pub is_public: i64,
}

pub async fn get_by_id(state: &AppState, id: i64) -> Option<NoteRecord> {
    query_as("SELECT * FROM notes WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap()
}

pub async fn get_by_ap_url(state: &AppState, ap_url: &str) -> Option<NoteRecord> {
    query_as("SELECT * FROM notes WHERE ap_url = $1")
        .bind(ap_url)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap()
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct NoteWithAuthorRecord {
    pub author_id: i64,
    pub display_name: String,
    pub username: String,
    pub id: i64,
    pub boosted_id: Option<i64>,
    pub boosted_username: Option<String>,
    pub boosted_created_at: Option<String>,
    pub content: String,
    pub attachments: Option<String>,
    pub parent_id: Option<i64>,
    pub parent_author_username: Option<String>,
    pub created_at: String,
    pub is_public: i64,
    pub like_count: i64,
    pub boost_count: i64,
}

pub async fn get_with_author_by_id(state: &AppState, id: i64) -> Option<NoteWithAuthorRecord> {
    query_as(
        "SELECT n.author_id, u.display_name, u.username, n.id, n.boosted_id, n.boosted_username, n.boosted_created_at, n.content, n.attachments, n.parent_id, n.parent_author_username, n.created_at, n.is_public, n.like_count, n.boost_count
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        WHERE n.id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap()
}

pub async fn get_replies_by_parent_id(
    state: &AppState,
    parent_id: i64,
) -> Vec<NoteWithAuthorRecord> {
    query_as(
        "SELECT n.author_id, u.display_name, u.username, n.id, n.boosted_id, n.boosted_username, n.boosted_created_at, n.content, n.attachments, n.parent_id, n.parent_author_username, n.created_at, n.is_public, n.like_count, n.boost_count
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        WHERE n.parent_id = $1
        AND n.is_public = 1
        AND n.boosted_id IS NULL
        ORDER BY n.created_at ASC",
    )
    .bind(parent_id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

pub async fn create(
    state: &AppState,
    id: i64,
    ap_url: &str,
    author_id: i64,
    content: &str,
    attachments: Option<String>,
    parent_id: Option<i64>,
    parent_author_username: Option<String>,
    created_at: &str,
    is_public: i64,
) {
    for attempt in 0..10 {
        let res = query(
            "INSERT INTO notes (id, ap_url, author_id, content, attachments, parent_id, parent_author_username, created_at, is_public)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(id + attempt)
        .bind(ap_url)
        .bind(author_id)
        .bind(content)
        .bind(&attachments)
        .bind(parent_id)
        .bind(&parent_author_username)
        .bind(created_at)
        .bind(is_public)
        .execute(&state.db_pool)
        .await;

        if res.is_ok() {
            return;
        } else {
            println!(
                "Failed to insert note with id {}: {:?}",
                id + attempt,
                res.err()
            );
        }
    }
    panic!("Failed to insert note");
}

pub async fn increment_like_count(state: &AppState, id: i64) {
    query(
        "UPDATE notes
        SET like_count = like_count + 1
        WHERE id = $1",
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn decrement_like_count(state: &AppState, id: i64) {
    query(
        "UPDATE notes
        SET like_count = like_count - 1
        WHERE id = $1 AND like_count > 0",
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn increment_boost_count(state: &AppState, id: i64) {
    query(
        "UPDATE notes
        SET boost_count = boost_count + 1
        WHERE id = $1",
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn decrement_boost_count(state: &AppState, id: i64) {
    query(
        "UPDATE notes
        SET boost_count = boost_count - 1
        WHERE id = $1 AND boost_count > 0",
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn delete(state: &AppState, id: i64) {
    query(
        "DELETE FROM notes
        WHERE id = $1",
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

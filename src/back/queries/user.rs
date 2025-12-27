use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: Option<String>,
    pub ap_url: String,
    pub inbox_url: String,
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub display_name: String,
    pub bio: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_local: i64,
}
pub async fn get_by_username(state: &AppState, username: &str) -> Option<UserRecord> {
    query_as("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap()
}

pub async fn get_by_id(state: &AppState, id: i64) -> UserRecord {
    query_as("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db_pool)
        .await
        .unwrap()
}

pub async fn create(
    state: &AppState,
    username: &str,
    password_hash: &str,
    ap_url: &str,
    inbox_url: &str,
    private_key: &str,
    public_key: &str,
    display_name: &str,
    bio: &str,
    created_at: &str,
    updated_at: &str,
    is_local: i64,
) {
    query(
        "INSERT INTO users (username, password_hash, ap_url, inbox_url, private_key, public_key, display_name, bio, created_at, updated_at, is_local)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(username)
    .bind(password_hash)
    .bind(ap_url)
    .bind(inbox_url)
    .bind(private_key)
    .bind(public_key)
    .bind(display_name)
    .bind(bio)
    .bind(created_at)
    .bind(updated_at)
    .bind(is_local)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn update_date(state: &AppState, id: i64, updated_at: &str) {
    query("UPDATE users SET updated_at = ? WHERE id = ?")
        .bind(updated_at)
        .bind(id)
        .execute(&state.db_pool)
        .await
        .unwrap();
}

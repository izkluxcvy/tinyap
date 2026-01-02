use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow, serde::Serialize)]
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
    pub note_count: i64,
    pub following_count: i64,
    pub follower_count: i64,
}
pub async fn get_by_username(state: &AppState, username: &str) -> Option<UserRecord> {
    query_as("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap()
}

pub async fn get_by_ap_url(state: &AppState, ap_url: &str) -> Option<UserRecord> {
    query_as("SELECT * FROM users WHERE ap_url = ?")
        .bind(ap_url)
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
    password_hash: Option<&str>,
    ap_url: &str,
    inbox_url: &str,
    private_key: Option<&str>,
    public_key: Option<&str>,
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

#[derive(sqlx::FromRow)]
pub struct TempSignUserRecord {
    pub ap_url: String,
    pub private_key: Option<String>,
}
pub async fn get_temp_sign_user(state: &AppState) -> TempSignUserRecord {
    query_as("SELECT ap_url, private_key FROM users WHERE is_local = 1 LIMIT 1")
        .fetch_one(&state.db_pool)
        .await
        .unwrap()
}

pub async fn update_profile(state: &AppState, ap_url: &str, display_name: &str, bio: &str) {
    query("UPDATE users SET display_name = ?, bio = ? WHERE ap_url = ?")
        .bind(display_name)
        .bind(bio)
        .bind(ap_url)
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

pub async fn increment_note_count(state: &AppState, id: i64) {
    query("UPDATE users SET note_count = note_count + 1 WHERE id = ?")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .unwrap();
}

pub async fn decrement_note_count(state: &AppState, id: i64) {
    query("UPDATE users SET note_count = note_count - 1 WHERE id = ? AND note_count > 0")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .unwrap();
}

pub async fn increment_following_count(state: &AppState, id: i64) {
    query("UPDATE users SET following_count = following_count + 1 WHERE id = ?")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .unwrap();
}

pub async fn decrement_following_count(state: &AppState, id: i64) {
    query("UPDATE users SET following_count = following_count - 1 WHERE id = ? AND following_count > 0")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .unwrap();
}

pub async fn increment_follower_count(state: &AppState, id: i64) {
    query("UPDATE users SET follower_count = follower_count + 1 WHERE id = ?")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .unwrap();
}

pub async fn decrement_follower_count(state: &AppState, id: i64) {
    query(
        "UPDATE users SET follower_count = follower_count - 1 WHERE id = ? AND follower_count > 0",
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

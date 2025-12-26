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
    pub display_name: Option<String>,
    pub bio: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_local: i64,
}
pub async fn get_user_by_username(state: &AppState, username: &str) -> Option<UserRecord> {
    query_as("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap()
}

pub async fn create_user(
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

pub async fn create_session(state: &AppState, session_id: &str, user_id: &i64, expires_at: &str) {
    query(
        "INSERT INTO sessions (session_id, user_id, expires_at)
        VALUES (?, ?, ?)",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(expires_at)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn delete_old_sessions(
    state: &AppState,
    user_id: &i64,
    max_sessions: &i64,
    date_now: &str,
) {
    query(
        "DELETE FROM sessions
        WHERE (session_id IN (
            SELECT session_id FROM sessions
            WHERE user_id = $1
            ORDER BY expires_at ASC
            LIMIT 1
        )
        AND (SELECT COUNT(*) FROM sessions WHERE user_id = $1) > $2)
        OR expires_at < $3",
    )
    .bind(user_id)
    .bind(max_sessions)
    .bind(date_now)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

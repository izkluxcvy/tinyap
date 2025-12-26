use crate::back::init::AppState;

use sqlx::query;

pub async fn get_userid_by_username(state: &AppState, username: &str) -> Option<i64> {
    let row = query!("SELECT id FROM users WHERE username = ?", username)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap();

    if let Some(row) = row { row.id } else { None }
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
    query!(
        "INSERT INTO users (username, password_hash, ap_url, inbox_url, private_key, public_key, display_name, bio, created_at, updated_at, is_local)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        username,
        password_hash,
        ap_url,
        inbox_url,
        private_key,
        public_key,
        display_name,
        bio,
        created_at,
        updated_at,
        is_local,
    )
    .execute(&state.db_pool)
    .await
    .unwrap();
}

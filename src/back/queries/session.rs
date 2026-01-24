use crate::back::init::AppState;

use sqlx::{query, query_as};

pub async fn create(state: &AppState, session_id: &str, user_id: &i64, expires_at: &str) {
    query(
        "INSERT INTO sessions (session_id, user_id, expires_at)
        VALUES ($1, $2, $3)",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(expires_at)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn delete_old(state: &AppState, user_id: &i64, max_sessions: &i64, date_now: &str) {
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

pub async fn delete_by_session_id(state: &AppState, session_id: &str) {
    query("DELETE FROM sessions WHERE session_id = $1")
        .bind(session_id)
        .execute(&state.db_pool)
        .await
        .unwrap();
}

pub async fn delete_by_user_id(state: &AppState, user_id: i64) {
    query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&state.db_pool)
        .await
        .unwrap();
}

#[derive(sqlx::FromRow)]
pub struct SessionRecord {
    pub user_id: i64,
}
pub async fn get(state: &AppState, session_id: &str, date_now: &str) -> Option<SessionRecord> {
    query_as(
        "SELECT user_id FROM sessions
        WHERE session_id = $1
        AND expires_at > $2",
    )
    .bind(session_id)
    .bind(date_now)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap()
}

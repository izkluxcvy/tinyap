use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow)]
pub struct LikeRecord {
    pub _id: i64,
}

pub async fn get(state: &AppState, user_id: i64, note_id: i64) -> Option<LikeRecord> {
    query_as(
        "SELECT id AS _id FROM likes
        WHERE user_id = $1
        AND note_id = $2",
    )
    .bind(user_id)
    .bind(note_id)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap()
}

pub async fn create(state: &AppState, user_id: i64, note_id: i64) {
    query(
        "INSERT INTO likes (user_id, note_id)
        VALUES ($1, $2)",
    )
    .bind(user_id)
    .bind(note_id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn delete(state: &AppState, user_id: i64, note_id: i64) {
    query(
        "DELETE FROM likes
        WHERE user_id = $1
        AND note_id = $2",
    )
    .bind(user_id)
    .bind(note_id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow)]
pub struct FollowRecord {
    pub id: i64,
    pub follower_id: i64,
    pub followee_id: i64,
    pub pending: i64,
}

pub async fn get(state: &AppState, follower_id: i64, followee_id: i64) -> Option<FollowRecord> {
    query_as(
        "SELECT * FROM follows
        WHERE follower_id = ?
        AND followee_id = ?",
    )
    .bind(follower_id)
    .bind(followee_id)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap()
}

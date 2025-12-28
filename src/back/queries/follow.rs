use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow)]
pub struct FollowRecord {
    pub follower_id: i64,
    pub followee_id: i64,
    pub pending: i64,
}

pub async fn get(state: &AppState, follower_id: i64, followee_id: i64) -> Option<FollowRecord> {
    query_as(
        "SELECT follower_id, followee_id, pending FROM follows
        WHERE follower_id = ?
        AND followee_id = ?",
    )
    .bind(follower_id)
    .bind(followee_id)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap()
}

#[derive(sqlx::FromRow)]
pub struct FollowerRecord {
    pub inbox_url: String,
}

pub async fn get_followers(state: &AppState, followee_id: i64) -> Vec<FollowerRecord> {
    query_as(
        "SELECT users.inbox_url
        FROM follows
        JOIN users ON follows.follower_id = users.id
        WHERE follows.followee_id = ?
        AND follows.pending = 0",
    )
    .bind(followee_id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

pub async fn create(state: &AppState, follower_id: i64, followee_id: i64) {
    query(
        "INSERT INTO follows (follower_id, followee_id, pending)
        VALUES (?, ?, 1)",
    )
    .bind(follower_id)
    .bind(followee_id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn accept(state: &AppState, follower_id: i64, followee_id: i64) {
    query(
        "UPDATE follows SET pending = 0
        WHERE follower_id = ?
        AND followee_id = ?",
    )
    .bind(follower_id)
    .bind(followee_id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn delete(state: &AppState, follower_id: i64, followee_id: i64) {
    query(
        "DELETE FROM follows
        WHERE follower_id = ?
        AND followee_id = ?",
    )
    .bind(follower_id)
    .bind(followee_id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

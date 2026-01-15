use crate::back::init::AppState;

use sqlx::{query, query_as};

#[derive(sqlx::FromRow)]
pub struct FollowRecord {
    pub pending: i64,
}

pub async fn get(state: &AppState, follower_id: i64, followee_id: i64) -> Option<FollowRecord> {
    query_as(
        "SELECT follower_id, followee_id, pending FROM follows
        WHERE follower_id = $1
        AND followee_id = $2",
    )
    .bind(follower_id)
    .bind(followee_id)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap()
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct FollowUserRecord {
    pub display_name: String,
    pub username: String,
}

pub async fn get_following(
    state: &AppState,
    follower_id: i64,
    max_username: &str,
    limit: i64,
) -> Vec<FollowUserRecord> {
    query_as(
        "SELECT users.display_name, users.username
        FROM follows
        JOIN users ON follows.followee_id = users.id
        WHERE users.username > $1
        AND follows.follower_id = $2
        AND follows.pending = 0
        ORDER BY users.username ASC
        LIMIT $3",
    )
    .bind(max_username)
    .bind(follower_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

#[cfg(feature = "api")]
pub async fn get_following_in(
    state: &AppState,
    follower_id: i64,
    followee_usernames: &Vec<String>,
) -> Vec<FollowUserRecord> {
    let in_placeholder = (1..followee_usernames.len())
        .map(|id| format!("${}", id + 1))
        .collect::<Vec<String>>()
        .join(", ");
    let query_str = format!(
        "SELECT users.display_name, users.username
        FROM follows
        JOIN users ON follows.followee_id = users.id
        WHERE follows.follower_id = $1
        AND users.username IN ({}) 
        AND follows.pending = 0",
        in_placeholder
    );
    let mut query = query_as(&query_str).bind(follower_id);
    for username in followee_usernames {
        query = query.bind(username);
    }

    query.fetch_all(&state.db_pool).await.unwrap()
}

pub async fn get_followers(
    state: &AppState,
    followee_id: i64,
    max_username: &str,
    limit: i64,
) -> Vec<FollowUserRecord> {
    query_as(
        "SELECT users.display_name, users.username
        FROM follows
        JOIN users ON follows.follower_id = users.id
        WHERE users.username > $1
        AND follows.followee_id = $2
        AND follows.pending = 0
        ORDER BY users.username ASC
        LIMIT $3",
    )
    .bind(max_username)
    .bind(followee_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

#[cfg(feature = "api")]
pub async fn get_followers_in(
    state: &AppState,
    followee_id: i64,
    follower_usernames: &Vec<String>,
) -> Vec<FollowUserRecord> {
    let in_placeholder = (1..follower_usernames.len())
        .map(|id| format!("${}", id + 1))
        .collect::<Vec<String>>()
        .join(", ");
    let query_str = format!(
        "SELECT users.display_name, users.username
        FROM follows
        JOIN users ON follows.follower_id = users.id
        WHERE follows.followee_id = $1
        AND users.username IN ({}) 
        AND follows.pending = 0",
        in_placeholder
    );
    let mut query = query_as(&query_str).bind(followee_id);
    for username in follower_usernames {
        query = query.bind(username);
    }

    query.fetch_all(&state.db_pool).await.unwrap()
}

#[derive(sqlx::FromRow)]
pub struct FollowerInboxRecord {
    pub inbox_url: String,
}

pub async fn get_follower_inboxes(state: &AppState, followee_id: i64) -> Vec<FollowerInboxRecord> {
    query_as(
        "SELECT users.inbox_url
        FROM follows
        JOIN users ON follows.follower_id = users.id
        WHERE follows.followee_id = $1
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
        VALUES ($1, $2, 1)",
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
        WHERE follower_id = $1
        AND followee_id = $2",
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
        WHERE follower_id = $1
        AND followee_id = $2",
    )
    .bind(follower_id)
    .bind(followee_id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

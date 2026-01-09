use crate::back::init::AppState;
use crate::back::queries::note::NoteWithAuthorRecord;

use sqlx::query_as;

pub async fn get_user(
    state: &AppState,
    user_id: i64,
    until_date: &str,
    until_id: i64,
    limit: i64,
) -> Vec<NoteWithAuthorRecord> {
    query_as(
        "SELECT n.author_id, u.display_name, u.username, n.id, n.boosted_id, n.boosted_username, n.boosted_created_at, n.content, n.attachments, n.parent_id, n.parent_author_username, n.created_at, n.is_public, n.like_count, n.boost_count
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        WHERE ((n.created_at < $1)
        OR (n.created_at = $1 AND n.id <= $2))
        AND u.id = $3
        AND n.is_public = 1
        ORDER BY n.created_at DESC, n.id DESC
        LIMIT $4",
    )
    .bind(until_date)
    .bind(until_id)
    .bind(user_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

pub async fn get_home(
    state: &AppState,
    user_id: i64,
    until_date: &str,
    until_id: i64,
    limit: i64,
) -> Vec<NoteWithAuthorRecord> {
    query_as(
        "SELECT n.author_id, u.display_name, u.username, n.id, n.boosted_id, n.boosted_username, n.boosted_created_at, n.content, n.attachments, n.parent_id, n.parent_author_username, n.created_at, n.is_public, n.like_count, n.boost_count
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        LEFT JOIN follows AS f ON f.followee_id = u.id
        AND f.follower_id = $1
        WHERE ((n.created_at < $2)
        OR (n.created_at = $2 AND n.id <= $3))
        AND (f.follower_id = $1 OR u.id = $1)
        ORDER BY n.created_at DESC, n.id DESC
        LIMIT $4",
    )
    .bind(user_id)
    .bind(until_date)
    .bind(until_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

pub async fn get_local(
    state: &AppState,
    until_date: &str,
    until_id: i64,
    limit: i64,
) -> Vec<NoteWithAuthorRecord> {
    query_as(
        "SELECT n.author_id, u.display_name, u.username, n.id, n.boosted_id, n.boosted_username, n.boosted_created_at, n.content, n.attachments, n.parent_id, n.parent_author_username, n.created_at, n.is_public, n.like_count, n.boost_count
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        WHERE ((n.created_at < $1)
        OR (n.created_at = $1 AND n.id <= $2))
        AND u.is_local = 1
        AND n.is_public = 1
        ORDER BY n.created_at DESC, n.id DESC
        LIMIT $3",
    )
    .bind(until_date)
    .bind(until_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

pub async fn get_federated(
    state: &AppState,
    until_date: &str,
    until_id: i64,
    limit: i64,
) -> Vec<NoteWithAuthorRecord> {
    query_as(
        "SELECT n.author_id, u.display_name, u.username, n.id, n.boosted_id, n.boosted_username, n.boosted_created_at, n.content, n.attachments, n.parent_id, n.parent_author_username, n.created_at, n.is_public, n.like_count, n.boost_count
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        WHERE ((n.created_at < $1)
        OR (n.created_at = $1 AND n.id <= $2))
        AND n.is_public = 1
        ORDER BY n.created_at DESC, n.id DESC
        LIMIT $3",
    )
    .bind(until_date)
    .bind(until_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

#[cfg(feature = "api")]
pub async fn get_home_since(
    state: &AppState,
    user_id: i64,
    since_date: &str,
    since_id: i64,
    limit: i64,
) -> Vec<NoteWithAuthorRecord> {
    query_as(
        "SELECT n.author_id, u.display_name, u.username, n.id, n.boosted_id, n.boosted_username, n.boosted_created_at, n.content, n.attachments, n.parent_id, n.parent_author_username, n.created_at, n.is_public, n.like_count, n.boost_count
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        LEFT JOIN follows AS f ON f.followee_id = u.id
        AND f.follower_id = $1
        WHERE ((n.created_at > $2)
        OR (n.created_at = $2 AND n.id > $3))
        AND (f.follower_id = $1 OR u.id = $1)
        ORDER BY n.created_at DESC, n.id DESC
        LIMIT $4",
    )
    .bind(user_id)
    .bind(since_date)
    .bind(since_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

#[cfg(feature = "api")]
pub async fn get_local_since(
    state: &AppState,
    since_date: &str,
    since_id: i64,
    limit: i64,
) -> Vec<NoteWithAuthorRecord> {
    query_as(
        "SELECT n.author_id, u.display_name, u.username, n.id, n.boosted_id, n.boosted_username, n.boosted_created_at, n.content, n.attachments, n.parent_id, n.parent_author_username, n.created_at, n.is_public, n.like_count, n.boost_count
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        WHERE ((n.created_at > $1)
        OR (n.created_at = $1 AND n.id > $2))
        AND u.is_local = 1
        AND n.is_public = 1
        ORDER BY n.created_at DESC, n.id DESC
        LIMIT $3",
    )
    .bind(since_date)
    .bind(since_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

#[cfg(feature = "api")]
pub async fn get_federated_since(
    state: &AppState,
    since_date: &str,
    since_id: i64,
    limit: i64,
) -> Vec<NoteWithAuthorRecord> {
    query_as(
        "SELECT n.author_id, u.display_name, u.username, n.id, n.boosted_id, n.boosted_username, n.boosted_created_at, n.content, n.attachments, n.parent_id, n.parent_author_username, n.created_at, n.is_public, n.like_count, n.boost_count
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        WHERE ((n.created_at > $1)
        OR (n.created_at = $1 AND n.id > $2))
        AND n.is_public = 1
        ORDER BY n.created_at DESC, n.id DESC
        LIMIT $3",
    )
    .bind(since_date)
    .bind(since_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

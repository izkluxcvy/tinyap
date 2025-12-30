use crate::back::init::AppState;
use crate::back::queries::note::NoteWithAuthorRecord;

use sqlx::query_as;

pub async fn get_user(state: &AppState, user_id: i64, limit: i64) -> Vec<NoteWithAuthorRecord> {
    query_as(
        "SELECT u.display_name, u.username, n.id, n.boosted_id, n.boosted_username, n.boosted_created_at, n.content, n.attachments, n.parent_id, n.parent_author_username, n.created_at, n.is_public, n.like_count, n.boost_count
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        WHERE u.id = ?
        AND n.is_public = 1
        ORDER BY n.created_at DESC
        LIMIT ?",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

pub async fn get_local(state: &AppState, limit: i64) -> Vec<NoteWithAuthorRecord> {
    query_as(
        "SELECT u.display_name, u.username, n.id, n.boosted_id, n.boosted_username, n.boosted_created_at, n.content, n.attachments, n.parent_id, n.parent_author_username, n.created_at, n.is_public, n.like_count, n.boost_count
        FROM notes AS n
        JOIN users AS u ON n.author_id = u.id
        WHERE u.is_local = 1
        AND n.is_public = 1
        ORDER BY n.created_at DESC
        LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

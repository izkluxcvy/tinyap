use crate::ap::utils;
use crate::state::AppState;
use crate::user::create_remoteuser;

use serde_json::Value;

pub async fn process(activity: &Value, state: &AppState) {
    let actor = activity["actor"].as_str().unwrap(); // : Remote User
    let object = activity["object"].as_str().unwrap(); // : Local Note

    sqlx::query!(
        "INSERT OR IGNORE INTO likes (note_apid, actor)
        VALUES (?, ?)",
        object,
        actor
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    sqlx::query!(
        "UPDATE notes SET like_count = like_count + 1 WHERE ap_id = ?",
        object
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    // Add notification
    let row = sqlx::query!(
        "SELECT users.username, notes.uuid
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.ap_id = ?",
        object
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    // Create local user if not exists
    let local_user = sqlx::query!("SELECT id FROM users WHERE actor_id = ?", actor)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap();

    if local_user.is_none() {
        create_remoteuser(actor, state).await;
    }

    let local_user = sqlx::query!("SELECT username FROM users WHERE actor_id = ?", actor)
        .fetch_one(&state.db_pool)
        .await
        .unwrap();

    utils::add_notification(
        &row.username,
        "like",
        &local_user.username,
        Some(&row.uuid),
        state,
    )
    .await;
}

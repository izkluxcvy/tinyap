use crate::ap::utils;
use crate::note::create_remotenote;
use crate::state::AppState;

use serde_json::Value;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub async fn process(activity: &Value, state: &AppState) {
    let actor = activity["actor"].as_str().unwrap();
    let object = activity["object"].as_str().unwrap();

    create_remotenote(object, state).await;

    let actor_user = sqlx::query!("SELECT id, username FROM users WHERE actor_id = ?", actor)
        .fetch_one(&state.db_pool)
        .await
        .unwrap();

    let note = sqlx::query!(
        "SELECT users.username, notes.uuid, notes.created_at, notes.content, notes.in_reply_to
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.ap_id = ?",
        object,
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    let boost_uuid = format!("{}-boost-{}", actor_user.username, note.uuid);
    let boost_apid = format!("{}-boost-{}", actor_user.username, object);
    let date_now = OffsetDateTime::now_utc().format(&Rfc3339).unwrap();
    sqlx::query!(
        "INSERT INTO notes (uuid, ap_id, user_id, boosted_username, boosted_created_at, content, in_reply_to, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        boost_uuid,
        boost_apid,
        actor_user.id,
        note.username,
        note.created_at,
        note.content,
        note.in_reply_to,
        date_now,
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    sqlx::query!(
        "UPDATE notes SET boost_count = boost_count + 1 WHERE ap_id = ?",
        object
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    // Add notification
    if object.starts_with(&format!("https://{}", state.domain)) {
        utils::add_notification(
            &note.username,
            "boost",
            &actor_user.username,
            Some(&note.uuid),
            state,
        )
        .await;
    }
}

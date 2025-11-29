use crate::ap::utils;
use crate::state::AppState;
use crate::user::{create_remoteuser, update_remoteuser};

use serde_json::Value;
use uuid::Uuid;

pub async fn note(activity: &Value, state: &AppState) {
    let note = &activity["object"];
    let note_apid = note["id"].as_str().unwrap();
    let existing = sqlx::query!("SELECT id FROM notes WHERE ap_id = ?", note_apid)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap();

    if existing.is_some() {
        return;
    }

    let note_actor = note["attributedTo"].as_str().unwrap();
    let existing_user = sqlx::query!("SELECT id FROM users WHERE actor_id = ?", note_actor)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap();
    if existing_user.is_none() {
        create_remoteuser(note_actor, state).await;
    } else {
        update_remoteuser(note_actor, state).await;
    }

    let user = sqlx::query!("SELECT id FROM users WHERE actor_id = ?", note_actor)
        .fetch_one(&state.db_pool)
        .await
        .unwrap();
    let uuid = Uuid::new_v4().to_string();
    let note_content = note["content"].as_str().unwrap();
    let content_clean = utils::strip_html_tags(note_content);
    let note_created_at = note["published"].as_str().unwrap();
    let note_inreplyto = note["inReplyTo"].as_str();
    sqlx::query!(
        "INSERT INTO notes (uuid, ap_id, user_id, content, in_reply_to, created_at)
        VALUES (?, ?, ?, ?, ?, ?)",
        uuid,
        note_apid,
        user.id,
        content_clean,
        note_inreplyto,
        note_created_at
    )
    .execute(&state.db_pool)
    .await
    .expect("Failed to insert note into database");

    if let Some(inreplyto) = note_inreplyto {
        let parent_note = sqlx::query!("SELECT user_id FROM notes WHERE ap_id = ?", inreplyto)
            .fetch_optional(&state.db_pool)
            .await
            .unwrap();

        if let Some(parent_note) = parent_note {
            let parent_user = sqlx::query!(
                "SELECT username FROM users WHERE id = ?",
                parent_note.user_id
            )
            .fetch_one(&state.db_pool)
            .await
            .unwrap();

            utils::add_notification(
                &parent_user.username,
                "reply",
                note_actor,
                Some(&uuid),
                state,
            )
            .await;
        }
    }
}

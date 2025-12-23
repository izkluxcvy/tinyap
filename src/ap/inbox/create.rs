use crate::ap::utils;
use crate::note::create_remotenote;
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

    let user = sqlx::query!(
        "SELECT id, username FROM users WHERE actor_id = ?",
        note_actor
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();
    let uuid = Uuid::new_v4().to_string();
    let mut note_content = note["content"].as_str().unwrap().to_string();
    if let Some(attachments) = note["attachment"].as_array() {
        note_content.push_str("\n");
        for attachment in attachments {
            if let Some(url) = attachment["url"].as_str() {
                note_content.push_str("\n");
                note_content.push_str(url);
            }
        }
    }
    let content_clean = utils::strip_html_tags(&note_content);
    let content_clean = if content_clean.chars().count() > state.config.max_note_chars {
        let byte_end = content_clean
            .char_indices()
            .nth(state.config.max_note_chars)
            .unwrap()
            .0;
        content_clean[..byte_end].to_string()
    } else {
        content_clean
    };
    let note_created_at = note["published"].as_str().unwrap();
    let note_inreplyto = note["inReplyTo"].as_str();
    let note_is_public = {
        let to_array = note["to"].as_array().unwrap();
        to_array
            .iter()
            .any(|v| v.as_str().unwrap() == "https://www.w3.org/ns/activitystreams#Public")
            as i32
    };

    // Reply to
    if let Some(inreplyto) = note_inreplyto {
        create_remotenote(inreplyto, state).await;
        let parent_note =
            sqlx::query!("SELECT user_id, uuid FROM notes WHERE ap_id = ?", inreplyto)
                .fetch_optional(&state.db_pool)
                .await
                .unwrap();

        if let Some(parent_note) = parent_note {
            let parent_user = sqlx::query!(
                "SELECT username FROM users WHERE id = ? AND is_local = 1",
                parent_note.user_id
            )
            .fetch_one(&state.db_pool)
            .await
            .unwrap();

            utils::add_notification(
                &parent_user.username,
                "reply",
                &user.username,
                Some(&parent_note.uuid),
                state,
            )
            .await;
        }
    }

    let reply_to_author = if let Some(in_reply_to) = note_inreplyto {
        let parent_note = sqlx::query!(
            "SELECT users.username FROM notes JOIN users ON notes.user_id = users.id WHERE notes.ap_id = ?",
            in_reply_to
        )
        .fetch_one(&state.db_pool)
        .await
        .unwrap();

        Some(parent_note.username)
    } else {
        None
    };

    let res = sqlx::query!(
        "INSERT INTO notes (uuid, ap_id, user_id, content, in_reply_to, reply_to_author, created_at, is_public)
        VALUES (?, ?, ?, ?, ?, ?, (strftime('%Y-%m-%dT%H:%M:%SZ', datetime(?, 'utc'))), ?)",
        uuid,
        note_apid,
        user.id,
        content_clean,
        note_inreplyto,
        reply_to_author,
        note_created_at,
        note_is_public,
    )
    .execute(&state.db_pool)
    .await;

    if let Err(e) = res {
        println!("maybe already inserted? {}", e);
        return;
    }
}

use crate::ap::utils;
use crate::auth::AuthUser;
use crate::state::AppState;
use crate::user;

use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect},
};
use serde_json::json;
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct NewNoteForm {
    pub content: String,
    pub in_reply_to: Option<String>,
}

pub async fn create_note(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<NewNoteForm>,
) -> impl IntoResponse {
    if form.content.chars().count() > state.config.max_note_chars {
        return Redirect::to("/?message=exceed_max_note_length");
    }

    let user = sqlx::query!(
        "SELECT id, username, private_key, is_local FROM users WHERE id = ?",
        user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    if &user.is_local == &0 {
        return Redirect::to("/");
    }

    let uuid = Uuid::new_v4().to_string();
    let ap_id = format!("https://{}/notes/{}", state.domain, uuid);
    let created_at = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    let content_trimed = form.content.trim();
    if content_trimed.is_empty() {
        return Redirect::to("/home");
    }
    sqlx::query!(
        "INSERT INTO notes (uuid, ap_id, user_id, content, in_reply_to, created_at)
        VALUES (?, ?, ?, ?, ?, ?)",
        uuid,
        ap_id,
        user.id,
        content_trimed,
        form.in_reply_to,
        created_at,
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    // Add notification
    let mut parent_actor: Option<String> = None;
    let mut parent_username: Option<String> = None;
    if let Some(in_reply_to) = &form.in_reply_to {
        let parent_note = sqlx::query!(
            "SELECT notes.user_id, notes.uuid, users.username, users.actor_id FROM notes
            JOIN users ON notes.user_id = users.id
            WHERE notes.ap_id = ?",
            in_reply_to
        )
        .fetch_optional(&state.db_pool)
        .await
        .unwrap();

        if let Some(parent_note) = parent_note {
            if parent_note.user_id != user.id {
                utils::add_notification(
                    &parent_note.username,
                    "reply",
                    &user.username,
                    Some(&parent_note.uuid),
                    &state,
                )
                .await;

                parent_actor = Some(parent_note.actor_id);
                parent_username = Some(parent_note.username);
            }
        }
    }

    // Deliver Create activity to followers
    let note_url = format!("https://{}/notes/{}", state.domain, uuid);
    let actor_url = format!("https://{}/users/{}", state.domain, user.username);
    let note_page_url = format!("https://{}/@{}/{}", state.domain, user.username, uuid);
    let mut note_json = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": note_url,
        "type": "Note",
        "attributedTo": actor_url,
        "content": form.content,
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "published": created_at,
        "url": note_page_url,
    });
    if let Some(in_reply_to) = &form.in_reply_to {
        note_json["inReplyTo"] = json!(in_reply_to);
        note_json["tag"] = json!([{
            "type": "Mention",
            "href": &parent_actor.clone().unwrap(),
            "name": &parent_username.unwrap(),
        }]);
    }
    let create_json = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("{}/#create-{}", actor_url, uuid),
        "type": "Create",
        "actor": actor_url,
        "object": note_json,
    });
    let json_body = serde_json::to_string(&create_json).unwrap();

    let followers = sqlx::query!("SELECT inbox FROM followers WHERE user_id = ?", user.id)
        .fetch_all(&state.db_pool)
        .await
        .unwrap();

    let private_key = user.private_key.unwrap();
    let mut already_delivered_hosts = vec![state.domain.clone()];
    for follower in followers {
        let url = Url::parse(&follower.inbox).unwrap();
        let host = url.host_str().expect("Invalid inbox URL").to_string();
        if !already_delivered_hosts.contains(&host) {
            utils::deliver_signed(&follower.inbox, &json_body, &private_key, &actor_url)
                .await
                .unwrap();
            already_delivered_hosts.push(host);
        }
    }

    // Deliver to parent note author
    if let Some(parent_actor) = &parent_actor
        && !parent_actor.starts_with(&format!("https://{}", state.domain))
    {
        let parent_inbox = utils::fetch_inbox(&parent_actor, &state).await;
        utils::deliver_signed(&parent_inbox.unwrap(), &json_body, &private_key, &actor_url)
            .await
            .unwrap();
    }

    Redirect::to("/home")
}

pub async fn create_remotenote(ap_id: &str, state: &AppState) {
    let existing = sqlx::query!("SELECT id FROM notes WHERE ap_id = ?", ap_id)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap();

    if existing.is_some() {
        return;
    }

    let res = utils::signed_get(ap_id, state).await.unwrap();

    let json: serde_json::Value = res.json().await.unwrap();
    if json["type"] != "Note" {
        return;
    }

    // Create local user if not exists
    let actor_url = json["attributedTo"].as_str().unwrap();
    let existing_user = sqlx::query!("SELECT id FROM users WHERE actor_id = ?", actor_url)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap();

    if existing_user.is_none() {
        user::create_remoteuser(actor_url, state).await;
    }

    let actor_user = sqlx::query!("SELECT id FROM users WHERE actor_id = ?", actor_url)
        .fetch_one(&state.db_pool)
        .await
        .unwrap();

    let uuid = Uuid::new_v4().to_string();
    let content = json["content"].as_str().unwrap();
    let content_clean = utils::strip_html_tags(content);
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
    let in_reply_to = json["inReplyTo"].as_str();
    let created_at = json["published"].as_str().unwrap();

    sqlx::query!(
        "INSERT INTO notes (uuid, ap_id, user_id, content, in_reply_to, created_at)
        VALUES (?, ?, ?, ?, ?, ?)",
        uuid,
        ap_id,
        actor_user.id,
        content_clean,
        in_reply_to,
        created_at,
    )
    .execute(&state.db_pool)
    .await
    .unwrap();
}

use crate::ap::utils;
use crate::auth::AuthUser;
use crate::state::AppState;

use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect},
};
use serde_json::json;
use url::Url;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct BoostForm {
    pub ap_id: String,
}
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub async fn boost(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<BoostForm>,
) -> impl IntoResponse {
    let actor_user = sqlx::query!(
        "SELECT actor_id, username, private_key FROM users WHERE id = ?",
        user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    let note = sqlx::query!(
        "SELECT users.username, users.actor_id, notes.uuid, notes.content, notes.in_reply_to, notes.reply_to_author, notes.created_at
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.ap_id = ?",
        form.ap_id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    let boost_uuid = format!("{}-boost-{}", actor_user.username, note.uuid);
    let boost_apid = format!("{}-boost-{}", actor_user.username, form.ap_id);
    let date_now = OffsetDateTime::now_utc().format(&Rfc3339).unwrap();
    sqlx::query!(
        "INSERT INTO notes (uuid, ap_id, user_id, boosted_username, boosted_created_at, content, in_reply_to, reply_to_author, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        boost_uuid,
        boost_apid,
        user.id,
        note.username,
        note.created_at,
        note.content,
        note.in_reply_to,
        note.reply_to_author,
        date_now,
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    sqlx::query!(
        "UPDATE notes SET boost_count = boost_count + 1 WHERE ap_id = ?",
        form.ap_id
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    // Deliver Announce activity to followers
    let boost_json = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("{}#announce-{}", actor_user.actor_id, Uuid::new_v4()),
        "type": "Announce",
        "actor": actor_user.actor_id,
        "object": form.ap_id,
        "to": ["https://www.w3.org/ns/activitystreams#Public"]
    });

    let json_body = serde_json::to_string(&boost_json).unwrap();
    let private_key = actor_user.private_key.unwrap();

    let followers = sqlx::query!("SELECT inbox FROM followers WHERE user_id = ?", user.id)
        .fetch_all(&state.db_pool)
        .await
        .unwrap();

    let mut already_delivered_hosts = vec![state.domain.clone()];
    for follower in followers {
        let url = Url::parse(&follower.inbox).unwrap();
        let host = url.host_str().expect("Invalid inbox URL").to_string();
        if !already_delivered_hosts.contains(&host) {
            utils::deliver_signed(
                &follower.inbox,
                &json_body,
                &private_key,
                &actor_user.actor_id,
                &state,
            )
            .await
            .unwrap();
            already_delivered_hosts.push(host);
        }
    }

    // Deliver to original note author
    if !note
        .actor_id
        .starts_with(&format!("https://{}", state.domain))
    {
        let parent_inbox = utils::fetch_inbox(&note.actor_id, &state).await;
        utils::deliver_signed(
            &parent_inbox.unwrap(),
            &json_body,
            &private_key,
            &actor_user.actor_id,
            &state,
        )
        .await
        .unwrap();
    }

    // Add notification
    if form.ap_id.starts_with(&format!("https://{}", state.domain)) {
        utils::add_notification(
            &note.username,
            "boost",
            &actor_user.username,
            Some(&note.uuid),
            &state,
        )
        .await;
    }

    Redirect::to(&format!("/@{}/{}", note.username, note.uuid))
}

#[derive(serde::Deserialize)]
pub struct UnboostForm {
    pub ap_id: String,
}

pub async fn unboost(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<UnboostForm>,
) -> impl IntoResponse {
    let actor_user = sqlx::query!(
        "SELECT username, actor_id, private_key FROM users WHERE id = ?",
        user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    let note = sqlx::query!(
        "SELECT users.username, users.actor_id, notes.uuid
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.ap_id = ?",
        form.ap_id,
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    let boost_apid = format!("{}-boost-{}", actor_user.username, form.ap_id);
    sqlx::query!(
        "DELETE FROM notes WHERE ap_id = ? AND user_id = ?",
        boost_apid,
        user.id
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    sqlx::query!(
        "UPDATE notes SET boost_count = boost_count - 1 WHERE ap_id = ?",
        form.ap_id
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    // Deliver Undo Announce activity to followers
    let undo_json = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("{}#undo-announce-{}", actor_user.actor_id, Uuid::new_v4()),
        "type": "Undo",
        "actor": actor_user.actor_id,
        "object": {
            "type": "Announce",
            "actor": actor_user.actor_id,
            "object": form.ap_id,
        }
    });

    let json_body = serde_json::to_string(&undo_json).unwrap();
    let private_key = actor_user.private_key.unwrap();

    let followers = sqlx::query!("SELECT inbox FROM followers WHERE user_id = ?", user.id)
        .fetch_all(&state.db_pool)
        .await
        .unwrap();

    let mut already_delivered_hosts = vec![state.domain.clone()];
    for follower in followers {
        let url = Url::parse(&follower.inbox).unwrap();
        let host = url.host_str().expect("Invalid inbox URL").to_string();
        if !already_delivered_hosts.contains(&host) {
            utils::deliver_signed(
                &follower.inbox,
                &json_body,
                &private_key,
                &actor_user.actor_id,
                &state,
            )
            .await
            .unwrap();
            already_delivered_hosts.push(host);
        }
    }

    // Deliver to original note author
    if !note
        .actor_id
        .starts_with(&format!("https://{}", state.domain))
    {
        let parent_inbox = utils::fetch_inbox(&note.actor_id, &state).await;
        utils::deliver_signed(
            &parent_inbox.unwrap(),
            &json_body,
            &private_key,
            &actor_user.actor_id,
            &state,
        )
        .await
        .unwrap();
    }

    Redirect::to(&format!("/@{}/{}", note.username, note.uuid))
}

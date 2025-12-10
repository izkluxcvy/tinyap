use crate::ap::utils;
use crate::auth::AuthUser;
use crate::state::AppState;

use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect},
};
use serde_json::json;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct LikeForm {
    pub ap_id: String,
}

pub async fn like(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<LikeForm>,
) -> impl IntoResponse {
    let actor_user = sqlx::query!(
        "SELECT actor_id, username, private_key FROM users WHERE id = ?",
        user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();
    let private_key = actor_user.private_key.unwrap();

    let note = sqlx::query!(
        "SELECT users.username, users.actor_id, notes.uuid
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.ap_id = ?",
        form.ap_id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();
    let note_author = note.actor_id;
    let note_author_username = note.username;
    let note_uuid = note.uuid;

    sqlx::query!(
        "INSERT OR IGNORE INTO likes (note_apid, actor)
        VALUES (?, ?)",
        form.ap_id,
        actor_user.actor_id
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    sqlx::query!(
        "UPDATE notes SET like_count = like_count + 1 WHERE ap_id = ?",
        form.ap_id
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    if !note_author.starts_with(&format!("https://{}", state.domain)) {
        let like_json = json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": format!("{}#like-{}", actor_user.actor_id, Uuid::new_v4()),
            "type": "Like",
            "actor": actor_user.actor_id,
            "object": form.ap_id,
        });

        let inbox_url = utils::fetch_inbox(&note_author, &state).await.unwrap();
        let json_body = serde_json::to_string(&like_json).unwrap();

        let _ = utils::deliver_signed(
            &inbox_url,
            &json_body,
            &private_key,
            &actor_user.actor_id,
            &state,
        )
        .await;
    } else {
        utils::add_notification(
            &note_author_username,
            "like",
            &actor_user.username,
            Some(&note_uuid),
            &state,
        )
        .await;
    }

    Redirect::to(&format!("/@{}/{}", note_author_username, note_uuid))
}

#[derive(serde::Deserialize)]
pub struct UnlikeForm {
    pub ap_id: String,
}

pub async fn unlike(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<UnlikeForm>,
) -> impl IntoResponse {
    let actor_user = sqlx::query!(
        "SELECT actor_id, private_key FROM users WHERE id = ?",
        user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();
    let private_key = actor_user.private_key.unwrap();

    let note = sqlx::query!(
        "SELECT users.username, users.actor_id, notes.uuid
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.ap_id = ?",
        form.ap_id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();
    let note_author = note.actor_id;
    let note_author_username = note.username;
    let note_uuid = note.uuid;

    sqlx::query!(
        "DELETE FROM likes WHERE note_apid = ? AND actor = ?",
        form.ap_id,
        actor_user.actor_id
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    sqlx::query!(
        "UPDATE notes SET like_count = like_count - 1 WHERE ap_id = ?",
        form.ap_id
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    if !note_author.starts_with(&format!("https://{}", state.domain)) {
        let undo_json = json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "type": "Undo",
            "actor": actor_user.actor_id,
            "object": {
                "type": "Like",
                "actor": actor_user.actor_id,
                "object": form.ap_id,
            },
        });

        let inbox_url = utils::fetch_inbox(&note_author, &state).await.unwrap();
        let json_body = serde_json::to_string(&undo_json).unwrap();

        let _ = utils::deliver_signed(
            &inbox_url,
            &json_body,
            &private_key,
            &actor_user.actor_id,
            &state,
        )
        .await;
    }

    Redirect::to(&format!("/@{}/{}", note_author_username, note_uuid))
}

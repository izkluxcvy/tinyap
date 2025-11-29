use crate::ap::utils;
use crate::auth::AuthUser;
use crate::state::AppState;

use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect},
};
use serde_json::json;
use std::sync::Arc;
use tokio::task;

#[derive(serde::Deserialize)]
pub struct DeleteNoteForm {
    pub ap_id: String,
}

pub async fn note(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<DeleteNoteForm>,
) -> impl IntoResponse {
    let note_author = sqlx::query!(
        "SELECT users.id, users.actor_id, users.private_key, notes.uuid
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.ap_id = ?",
        form.ap_id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    if note_author.id != Some(user.id) {
        return Redirect::to("/").into_response();
    }

    sqlx::query!("DELETE FROM notes WHERE ap_id = ?", form.ap_id)
        .execute(&state.db_pool)
        .await
        .unwrap();

    let delete_json = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "type": "Delete",
        "actor": note_author.actor_id,
        "object": form.ap_id,
    });
    let json_body = serde_json::to_string(&delete_json).unwrap();

    let followers = sqlx::query!(
        "SELECT inbox FROM followers
        WHERE user_id = ?",
        note_author.id
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap();

    // Spawn background task to deliver delete activities
    let state = Arc::new(state);
    let json_body = json_body.clone();
    let actor_url = note_author.actor_id.clone();
    let private_key = note_author.private_key.clone().unwrap();

    task::spawn({
        let state = Arc::clone(&state);
        async move {
            for follower in followers {
                if !follower
                    .inbox
                    .starts_with(&format!("https://{}", state.domain))
                {
                    let _ = utils::deliver_signed(
                        &follower.inbox,
                        &json_body,
                        &private_key,
                        &actor_url,
                    )
                    .await;
                }
            }
        }
    });

    Redirect::to("/home").into_response()
}

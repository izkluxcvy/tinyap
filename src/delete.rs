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
        "id": format!("{}#delete-{}", note_author.actor_id, Uuid::new_v4()),
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

    let private_key = note_author.private_key.unwrap();
    let mut already_delivered_hosts = vec![state.domain.clone()];
    for follower in followers {
        let url = Url::parse(&follower.inbox).unwrap();
        let host = url.host_str().expect("Invalid inbox URL").to_string();
        if !already_delivered_hosts.contains(&host) {
            utils::deliver_signed(
                &follower.inbox,
                &json_body,
                &private_key,
                &note_author.actor_id,
                &state,
            )
            .await
            .unwrap();
            already_delivered_hosts.push(host);
        }
    }

    Redirect::to("/home").into_response()
}

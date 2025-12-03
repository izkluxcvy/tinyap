use crate::ap::utils;
use crate::auth::AuthUser;
use crate::state::AppState;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use serde_json::json;
use uuid::Uuid;

pub async fn follow(
    State(state): State<AppState>,
    user: AuthUser,
    Path(username): Path<String>,
) -> impl IntoResponse {
    let parts: Vec<&str> = username.split('@').collect();
    if parts.len() == 1 {
        follow_local(user.id, &username, &state).await;
        Redirect::to(&format!("/@{}", username))
    } else {
        follow_remote(user.id, &username, &state).await;
        Redirect::to(&format!("/@{}", username))
    }
}

async fn follow_local(user_id: i64, username: &str, state: &AppState) {
    let actor_user = sqlx::query!("SELECT username, actor_id FROM users WHERE id = ?", user_id)
        .fetch_one(&state.db_pool)
        .await
        .unwrap();

    let object_user = sqlx::query!(
        "SELECT id, actor_id FROM users WHERE username = ?",
        username
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    if object_user.id.unwrap() == user_id {
        return;
    }

    sqlx::query!(
        "INSERT INTO follows (user_id, object_actor, pending)
        VALUES (?, ?, 0)",
        user_id,
        object_user.actor_id,
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    let actor_inbox = format!(
        "https://{}/users/{}/inbox",
        state.domain, actor_user.username
    );
    sqlx::query!(
        "INSERT INTO followers (user_id, actor, inbox)
        VALUES (?, ?, ?)",
        object_user.id,
        actor_user.actor_id,
        actor_inbox
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    // Add notification
    utils::add_notification(username, "follow", &actor_user.username, None, state).await;
}

async fn follow_remote(user_id: i64, acct: &str, state: &AppState) {
    let object_user = sqlx::query!("SELECT id, actor_id FROM users WHERE username = ?", acct)
        .fetch_one(&state.db_pool)
        .await
        .unwrap();

    let inbox = match utils::fetch_inbox(&object_user.actor_id).await {
        Some(url) => url,
        None => return,
    };

    let actor_user = sqlx::query!(
        "SELECT username, actor_id, private_key FROM users WHERE id = ?",
        user_id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    let follow_id = format!("{}#follow-{}", actor_user.actor_id, Uuid::new_v4());

    let follow_json = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": follow_id,
        "type": "Follow",
        "actor": actor_user.actor_id,
        "object": object_user.actor_id,
    });

    let json_body = serde_json::to_string(&follow_json).unwrap();
    let private_key = &actor_user.private_key.unwrap();
    let _ = utils::deliver_signed(&inbox, &json_body, private_key, &actor_user.actor_id).await;

    sqlx::query!(
        "INSERT INTO follows (user_id, object_actor, pending)
        VALUES (?, ?, 1)",
        user_id,
        object_user.actor_id,
    )
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn unfollow(
    State(state): State<AppState>,
    user: AuthUser,
    Path(username): Path<String>,
) -> impl IntoResponse {
    let actor_user = sqlx::query!(
        "SELECT id, actor_id, private_key FROM users WHERE id = ?",
        user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    let object_user = sqlx::query!(
        "SELECT id, actor_id FROM users WHERE username = ?",
        username
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap();

    sqlx::query!(
        "DELETE FROM follows WHERE user_id = ? AND object_actor = ?",
        actor_user.id,
        object_user.actor_id
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    sqlx::query!(
        "DELETE FROM followers WHERE user_id = ? AND actor = ?",
        object_user.id,
        actor_user.actor_id
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    let parts: Vec<&str> = username.split('@').collect();
    if parts.len() == 2 {
        let inbox = match utils::fetch_inbox(&object_user.actor_id).await {
            Some(url) => url,
            None => return Redirect::to(&format!("/?message=inbox-not-found")),
        };

        let undo_id = format!("{}#undo-{}", actor_user.actor_id, Uuid::new_v4());

        let undo_json = json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": undo_id,
            "type": "Undo",
            "actor": actor_user.actor_id,
            "object": {
                "type": "Follow",
                "actor": actor_user.actor_id,
                "object": object_user.actor_id,
            }
        });
        let json_body = serde_json::to_string(&undo_json).unwrap();
        let private_key = &actor_user.private_key.unwrap();
        let _ = utils::deliver_signed(&inbox, &json_body, private_key, &actor_user.actor_id).await;
    }

    Redirect::to(&format!("/@{}", username))
}

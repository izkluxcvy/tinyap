use crate::ap::utils;
use crate::state::AppState;
use crate::user::create_remoteuser;

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::{Value, json};

pub async fn accept(
    user_id: i64,
    username: &str,
    activity: &Value,
    state: &AppState,
) -> impl IntoResponse {
    let actor = activity["actor"].as_str().unwrap(); // : Remote User
    let object = activity["object"].as_str().unwrap(); // : Local User

    if object != format!("https://{}/users/{}", state.domain, username) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid object in Follow activity"})),
        )
            .into_response();
    }

    let inbox_url = utils::fetch_inbox(actor).await.unwrap();
    sqlx::query!(
        "INSERT OR IGNORE INTO followers (user_id, actor, inbox) VALUES (?, ?, ?)",
        user_id,
        actor,
        inbox_url
    )
    .execute(&state.db_pool)
    .await
    .expect("Failed to insert follower");

    // Create remote user and insert into follows table
    create_remoteuser(actor, state).await;
    let actor_user = sqlx::query!("SELECT id, username FROM users WHERE actor_id = ?", actor)
        .fetch_one(&state.db_pool)
        .await
        .expect("Failed to fetch remote use");
    sqlx::query!(
        "INSERT OR IGNORE INTO follows (user_id, object_actor, pending) VALUES (?, ?, ?)",
        actor_user.id,
        object,
        0
    )
    .execute(&state.db_pool)
    .await
    .expect("Failed to insert follow");

    let accept_json = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("{}/#accept-follow-{}", object, uuid::Uuid::new_v4()),
        "type": "Accept",
        "actor": object,
        "object": activity,
    });
    let accept_str = accept_json.to_string();

    let row = sqlx::query!("SELECT private_key FROM users WHERE id = ?", user_id)
        .fetch_one(&state.db_pool)
        .await
        .expect("Failed to fetch user's private key");

    let private_key_pem = row.private_key.unwrap();
    utils::deliver_signed(&inbox_url, &accept_str, &private_key_pem, &object)
        .await
        .expect("Failed to deliver Accept activity");

    // Add notification
    utils::add_notification(username, "follow", &actor_user.username, None, state).await;

    (
        StatusCode::OK,
        Json(json!({"status": "Activity processed"})),
    )
        .into_response()
}

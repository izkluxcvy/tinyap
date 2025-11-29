use crate::state::AppState;

use serde_json::Value;

pub async fn follow(activity: Value, state: &AppState) {
    let actor = activity["object"]["actor"].as_str().unwrap(); // : Local User
    let object = activity["object"]["object"].as_str().unwrap(); // : Remote User

    let actor_user = sqlx::query!("SELECT id FROM users WHERE actor_id = ?", actor)
        .fetch_one(&state.db_pool)
        .await
        .unwrap();
    let object_user = sqlx::query!("SELECT id FROM users WHERE actor_id = ?", object)
        .fetch_one(&state.db_pool)
        .await
        .unwrap();

    sqlx::query!(
        "UPDATE follows SET pending = 0
        WHERE user_id = ? AND object_actor = ?",
        actor_user.id,
        object
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    let actor_inbox = format!(
        "https://{}/users/{}/inbox",
        state.domain,
        actor_user.id.unwrap()
    );
    sqlx::query!(
        "INSERT INTO followers (user_id, actor, inbox)
        VALUES(?, ?, ?)",
        object_user.id,
        actor,
        actor_inbox
    )
    .execute(&state.db_pool)
    .await
    .unwrap();
}

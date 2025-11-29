use crate::state::AppState;

use serde_json::Value;

pub async fn follow(activity: Value, state: &AppState) {
    let actor = activity["object"]["actor"].as_str().unwrap(); // : Remote User
    let object = activity["object"]["object"].as_str().unwrap(); // : Local User

    let actor_user = sqlx::query!("SELECT id FROM users WHERE actor_id = ?", actor)
        .fetch_one(&state.db_pool)
        .await
        .unwrap();
    let object_user = sqlx::query!("SELECT id FROM users WHERE actor_id = ?", object)
        .fetch_one(&state.db_pool)
        .await
        .unwrap();

    sqlx::query!(
        "DELETE FROM follows WHERE user_id = ? AND object_actor = ?",
        actor_user.id,
        object
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    sqlx::query!(
        "DELETE FROM followers WHERE user_id = ? AND actor = ?",
        object_user.id,
        actor
    )
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn like(activity: Value, state: &AppState) {
    let actor = activity["object"]["actor"].as_str().unwrap(); // : Remote User
    let object = activity["object"]["object"].as_str().unwrap(); // : Local Note

    sqlx::query!(
        "DELETE FROM likes WHERE note_apid = ? AND actor = ?",
        object,
        actor
    )
    .execute(&state.db_pool)
    .await
    .unwrap();
}

use crate::state::AppState;

// actor: Remote user who accepts the object's follow request
// object: Local user
pub async fn follow(actor: &str, object: &str, state: &AppState) {
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
        object_user.id,
        actor
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    let object_inbox = format!("{}/inbox", object);
    sqlx::query!(
        "INSERT INTO followers (user_id, actor, inbox)
        VALUES(?, ?, ?)",
        actor_user.id,
        object,
        object_inbox
    )
    .execute(&state.db_pool)
    .await
    .unwrap();
}

use crate::state::AppState;

pub async fn note(actor: &str, object: &str, state: &AppState) {
    let note_author = sqlx::query!(
        "SELECT users.actor_id, notes.uuid
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.ap_id = ?",
        object
    )
    .fetch_optional(&state.db_pool)
    .await
    .unwrap();

    let Some(note_author) = note_author else {
        return;
    };

    if note_author.actor_id != actor {
        return;
    }

    sqlx::query!("DELETE FROM notes WHERE ap_id = ?", object)
        .execute(&state.db_pool)
        .await
        .unwrap();
}

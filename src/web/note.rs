use crate::auth::MaybeAuthUser;
use crate::state::AppState;

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect},
};
use serde_json::json;

pub async fn page(
    State(state): State<AppState>,
    Path((username, uuid)): Path<(String, String)>,
    user: MaybeAuthUser,
) -> impl IntoResponse {
    let row = sqlx::query!(
        "SELECT notes.ap_id, notes.user_id, notes.content, notes.created_at, notes.in_reply_to, users.display_name, users.username, users.actor_id
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.uuid = ?
        AND users.username = ?",
        uuid,
        username
    )
    .fetch_optional(&state.db_pool)
    .await
    .unwrap();

    let Some(row) = row else {
        return Redirect::to("/local").into_response();
    };

    let note = json!({
        "uuid": uuid,
        "ap_id": row.ap_id,
        "display_name": row.display_name,
        "username": row.username,
        "content": row.content,
        "created_at": row.created_at,
    });

    // Like status
    let like_num = sqlx::query!(
        "SELECT COUNT(*) as count FROM likes WHERE note_apid = ?",
        row.ap_id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap()
    .count;

    let is_liked: bool;
    let is_you: bool;
    match user.id {
        Some(user_id) => {
            is_liked = sqlx::query!(
                "SELECT id FROM likes WHERE note_apid = ? AND actor = (SELECT actor_id FROM users WHERE id = ?)",
                row.ap_id,
                user_id
            )
            .fetch_optional(&state.db_pool)
            .await
            .unwrap()
            .is_some();

            is_you = row.user_id == user_id;
        }
        None => {
            is_liked = false;
            is_you = false;
        }
    }

    // Boost status
    // let boost_uuid = format!("%boost-{}", uuid);
    // let boost_num = sqlx::query!(
    //     "SELECT COUNT(*) as count
    //     FROM notes
    //     WHERE uuid LIKE ?",
    //     boost_uuid
    // )
    // .fetch_one(&state.db_pool)
    // .await
    // .unwrap()
    // .count;

    let is_boosted: bool;
    match user.id {
        Some(user_id) => {
            let user = sqlx::query!("SELECT username FROM users WHERE id = ?", user_id)
                .fetch_one(&state.db_pool)
                .await
                .unwrap();
            let boost_uuid = format!("{}-boost-{}", user.username, uuid);
            is_boosted = sqlx::query!("SELECT id FROM notes WHERE uuid = ?", boost_uuid,)
                .fetch_optional(&state.db_pool)
                .await
                .unwrap()
                .is_some();
        }
        None => {
            is_boosted = false;
        }
    }

    // Replies
    let row = sqlx::query!(
        "SELECT notes.uuid, notes.content, notes.created_at, notes.in_reply_to, users.display_name, users.username
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.ap_id = ?",
        row.in_reply_to
    )
    .fetch_optional(&state.db_pool)
    .await
    .unwrap();

    let reply_note: Option<serde_json::Value> = match row {
        Some(row) => Some(json!({
            "uuid": row.uuid,
            "display_name": row.display_name,
            "username": row.username,
            "content": row.content,
            "in_reply_to": row.in_reply_to,
            "created_at": row.created_at,
        })),
        None => None,
    };

    let note_apid = note["ap_id"].as_str().unwrap();
    let rows = sqlx::query!(
        "SELECT notes.uuid, notes.content, notes.created_at, users.display_name, users.username
        FROM notes
        JOIN users ON notes.user_id = users.id
        WHERE notes.in_reply_to = ?",
        note_apid
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap();

    let replies: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            json!({
                "uuid": row.uuid,
                "display_name": row.display_name,
                "username": row.username,
                "content": row.content,
                "created_at": row.created_at,
            })
        })
        .collect();

    let mut context = tera::Context::new();
    context.insert("site_name", &state.site_name);
    context.insert("note", &note);
    context.insert("reply_note", &reply_note);
    context.insert("timezone", &state.config.timezone);
    context.insert("is_liked", &is_liked);
    context.insert("like_num", &like_num);
    context.insert("is_boosted", &is_boosted);
    // context.insert("boost_num", &boost_num);
    context.insert("is_you", &is_you);
    context.insert("replies", &replies);
    let rendered = state.tera.render("note.html", &context).unwrap();

    Html(rendered).into_response()
}

use crate::auth::MaybeAuthUser;
use crate::state::AppState;
use crate::user::create_remoteuser;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};

pub async fn page(
    State(state): State<AppState>,
    Path(username): Path<String>,
    login_user: MaybeAuthUser,
) -> impl IntoResponse {
    let existing = sqlx::query!(
        "SELECT id
        FROM users WHERE username = ?",
        username
    )
    .fetch_optional(&state.db_pool)
    .await
    .unwrap();

    if existing.is_none() {
        let actor_url = resolve_acct(&username).await;
        if let Some(actor_url) = actor_url {
            create_remoteuser(&actor_url, &state).await;
        } else {
            return (StatusCode::NOT_FOUND, "User not found").into_response();
        }
    }

    let user = sqlx::query!(
        "SELECT id, actor_id, display_name, bio, created_at, is_local
        FROM users
        WHERE username = ?",
        username
    )
    .fetch_optional(&state.db_pool)
    .await
    .unwrap();

    let Some(user) = user else {
        return (StatusCode::NOT_FOUND, "User not found").into_response();
    };

    let rows = sqlx::query!(
        "SELECT uuid, content, created_at, in_reply_to
        FROM notes
        WHERE user_id = ?
        ORDER BY created_at DESC",
        user.id
    )
    .fetch_all(&state.db_pool)
    .await
    .unwrap();

    let notes: Vec<_> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "uuid": row.uuid,
                "content": row.content,
                "created_at": row.created_at,
                "in_reply_to": row.in_reply_to,
            })
        })
        .collect();

    let follow_status: String;
    match login_user.id {
        Some(id) => {
            let follow = sqlx::query!(
                "SELECT pending FROM follows
                WHERE user_id = ?
                AND object_actor = ?",
                id,
                user.actor_id
            )
            .fetch_optional(&state.db_pool)
            .await
            .unwrap();

            if let Some(follow) = follow {
                if follow.pending == 1 {
                    follow_status = "pending".to_string();
                } else {
                    follow_status = "following".to_string();
                }
            } else {
                if id == user.id.unwrap() {
                    follow_status = "self".to_string();
                } else {
                    follow_status = "to_follow".to_string();
                }
            }
        }
        None => {
            follow_status = "to_follow".to_string();
        }
    }

    let follow_num = sqlx::query!(
        "SELECT COUNT(*) as num FROM follows
        WHERE user_id = ?
        AND pending = 0",
        user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap()
    .num;

    let follower_num = sqlx::query!(
        "SELECT COUNT(*) as num FROM followers
        WHERE user_id = ?",
        user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .unwrap()
    .num;

    let mut context = tera::Context::new();
    context.insert("username", &username);
    context.insert("display_name", &user.display_name);
    context.insert("is_local", &user.is_local);
    context.insert("domain", &state.domain);
    context.insert("bio", &user.bio);
    context.insert("created_at", &user.created_at);
    context.insert("follow_num", &follow_num);
    context.insert("follower_num", &follower_num);
    context.insert("follow_status", &follow_status);
    context.insert("notes", &notes);
    let rendered = state.tera.render("user.html", &context).unwrap();
    Html(rendered).into_response()
}

async fn resolve_acct(acct: &str) -> Option<String> {
    let parts: Vec<&str> = acct.split('@').collect();
    if parts.len() != 2 {
        return None;
    }

    let domain = parts[1];

    let url = format!(
        "https://{}/.well-known/webfinger?resource=acct:{}",
        domain, acct
    );
    let client = reqwest::Client::new();

    let res_json: serde_json::Value = client
        .get(&url)
        .header("Accept", "application/jrd+json, application/json")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;

    let links = res_json["links"].as_array()?;
    for link in links {
        if link["rel"] == "self"
            && (link["type"] == "application/activity+json"
                || link["type"]
                    == "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")
        {
            return link["href"].as_str().map(|s| s.to_string());
        }
    }

    None
}

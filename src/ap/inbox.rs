mod accept;
mod create;
mod delete;
mod follow;
mod like;
mod undo;

use crate::state::AppState;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::{Value, json};

pub async fn api(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(activity): Json<Value>,
) -> impl IntoResponse {
    println!("Received activity for {}: {}", username, activity);
    let user = sqlx::query!("SELECT id FROM users WHERE username = ?", username)
        .fetch_optional(&state.db_pool)
        .await
        .expect("Failed to fetch user from database");

    let Some(user) = user else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        )
            .into_response();
    };

    match activity["actor"].as_str() {
        Some(actor) => {
            if actor.contains("localhost/") {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "Localhost not allowed"})),
                )
                    .into_response();
            }
        }
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "actor field missing"})),
            )
                .into_response();
        }
    }

    let user_id = user.id.unwrap();

    if activity["type"] == "Follow" {
        return follow::accept(user_id, &username, &activity, &state)
            .await
            .into_response();
    } else if activity["type"] == "Like" {
        like::process(&activity, &state).await;
    } else if activity["type"] == "Create" {
        if activity["object"]["type"] == "Note" {
            create::note(&activity, &state).await;
        }
    } else if activity["type"] == "Accept" {
        let actor = activity["actor"].as_str().unwrap();
        if activity["object"]["type"] == "Follow" {
            let object = activity["object"]["actor"].as_str().unwrap();
            accept::follow(actor, object, &state).await;
        } else if activity["object"].is_string() {
            let object_value = activity["object"].as_str().unwrap();
            let parts = object_value.split("#").collect::<Vec<&str>>();
            let object = parts[0];
            accept::follow(actor, object, &state).await;
        }
    } else if activity["type"] == "Undo" {
        if activity["object"]["type"] == "Follow" {
            undo::follow(activity, &state).await;
        } else if activity["object"]["type"] == "Like" {
            undo::like(activity, &state).await;
        }
    } else if activity["type"] == "Delete" {
        let actor = activity["actor"].as_str().unwrap();
        if activity["object"]["type"] == "Tombstone" {
            let object = activity["object"]["id"].as_str().unwrap();
            delete::note(actor, object, &state).await;
        } else if activity["object"].is_string() {
            let object = activity["object"].as_str().unwrap();
            delete::note(actor, object, &state).await;
        }
    }

    (
        StatusCode::OK,
        Json(json!({"status": "Activity processed"})),
    )
        .into_response()
}

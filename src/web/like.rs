use crate::back::init::AppState;
use crate::back::like;
use crate::back::queries;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

pub async fn post_like(
    State(state): State<AppState>,
    user: AuthUser,
    Path((username, id)): Path<(String, i64)>,
) -> impl IntoResponse {
    // Get note
    let Some(note) = queries::note::get_by_id(&state, id).await else {
        return "Note not found".into_response();
    };

    // Get author
    let author = queries::user::get_by_id(&state, note.author_id).await;

    // Like
    let res = like::like(&state, user.id, id).await;
    if let Err(e) = res {
        return e.into_response();
    };

    // Deliver like
    if author.is_local == 0 {
        like::deliver_like(&state, user.id, id).await;
    }

    Redirect::to(&format!("/@{}/{}", username, id)).into_response()
}

pub async fn post_unlike(
    State(state): State<AppState>,
    user: AuthUser,
    Path((username, id)): Path<(String, i64)>,
) -> impl IntoResponse {
    // Get note
    let Some(note) = queries::note::get_by_id(&state, id).await else {
        return "Note not found".into_response();
    };

    // Get author
    let author = queries::user::get_by_id(&state, note.author_id).await;

    // Unlike
    like::unlike(&state, user.id, id).await;

    // Deliver unlike
    if author.is_local == 0 {
        like::deliver_unlilke(&state, user.id, id).await;
    }

    Redirect::to(&format!("/@{}/{}", username, id)).into_response()
}

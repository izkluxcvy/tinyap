use crate::back::init::AppState;
use crate::back::note;
use crate::back::queries;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

pub async fn post(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_username, id)): Path<(String, i64)>,
) -> impl IntoResponse {
    // Get note
    let Some(note) = queries::note::get_by_id(&state, id).await else {
        return "Note not found".into_response();
    };

    // Check ownership
    if note.author_id != user.id {
        return "Unauthorized".into_response();
    }

    // Deliver delete activity
    note::deliver_delete(&state, id).await;

    // Delete
    note::delete(&state, id).await;

    Redirect::to("/home").into_response()
}

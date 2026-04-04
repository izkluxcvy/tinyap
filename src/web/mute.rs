use crate::back::init::AppState;
use crate::back::mute;
use crate::back::queries;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

pub async fn post_mute(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(mutee_username): Path<String>,
) -> impl IntoResponse {
    // Get mutee user
    let Some(mutee) = queries::user::get_by_username(&state, &mutee_username).await else {
        return "user not found".into_response();
    };

    // Mute
    let res = mute::mute(&state, auth_user.id, mutee.id).await;
    match res {
        Ok(_) => Redirect::to(&format!("/@{}", mutee_username)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn post_unmute(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(mutee_username): Path<String>,
) -> impl IntoResponse {
    // Get mutee user
    let Some(mutee) = queries::user::get_by_username(&state, &mutee_username).await else {
        return "user not found".into_response();
    };

    // Unmute
    let res = mute::unmute(&state, auth_user.id, mutee.id).await;
    match res {
        Ok(_) => Redirect::to(&format!("/@{}", mutee_username)).into_response(),
        Err(e) => e.into_response(),
    }
}

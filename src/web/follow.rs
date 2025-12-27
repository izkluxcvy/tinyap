use crate::back::follow;
use crate::back::init::AppState;
use crate::back::queries;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

pub async fn post_follow(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(followee_username): Path<String>,
) -> impl IntoResponse {
    // Get followee user
    let Some(followee) = queries::user::get_by_username(&state, &followee_username).await else {
        return "user not found".into_response();
    };

    // Follow
    let res = follow::follow(&state, auth_user.id, followee.id).await;
    match res {
        Ok(_) => {
            if followee.is_local == 1 {
                follow::approve(&state, auth_user.id, followee.id).await;
            } else {
                follow::deliver_follow(&state, auth_user.id, followee.id).await;
            }
            return Redirect::to(&format!("/@{}", followee_username)).into_response();
        }
        Err(e) => return e.into_response(),
    };
}

pub async fn post_unfollow(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(followee_username): Path<String>,
) -> impl IntoResponse {
    // Get followee user
    let Some(followee) = queries::user::get_by_username(&state, &followee_username).await else {
        return "user not found".into_response();
    };

    // Unfollow
    let res = follow::unfollow(&state, auth_user.id, followee.id).await;
    match res {
        Ok(_) => {
            return Redirect::to(&format!("/@{}", followee_username)).into_response();
        }
        Err(e) => return e.into_response(),
    };
}

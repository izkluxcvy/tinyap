use crate::back::boost;
use crate::back::init::AppState;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

pub async fn post_boost(
    State(state): State<AppState>,
    user: AuthUser,
    Path((username, id)): Path<(String, i64)>,
) -> impl IntoResponse {
    // Boost
    let res = boost::boost(&state, user.id, id).await;
    if let Err(e) = res {
        return e.into_response();
    };

    // Deliver to followers
    boost::deliver_boost(&state, user.id, id).await;

    Redirect::to(&format!("/@{}/{}", username, id)).into_response()
}

pub async fn post_unboost(
    State(state): State<AppState>,
    user: AuthUser,
    Path((username, id)): Path<(String, i64)>,
) -> impl IntoResponse {
    // Unboost
    let res = boost::unboost(&state, user.id, id).await;
    if let Err(e) = res {
        return e.into_response();
    };

    // Deliver to followers
    boost::deliver_unboost(&state, user.id, id).await;

    Redirect::to(&format!("/@{}/{}", username, id)).into_response()
}

use crate::back::init::AppState;
use crate::web::auth::AuthUser;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

pub async fn post_like(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
}

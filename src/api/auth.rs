use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

pub struct OAuthUser {
    pub id: i64,
}

impl FromRequestParts<AppState> for OAuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let date_now = utils::date_now();
        let Some(token) = queries::oauth::get_token(state, token, &date_now).await else {
            return Err(StatusCode::UNAUTHORIZED);
        };

        Ok(OAuthUser { id: token.user_id })
    }
}

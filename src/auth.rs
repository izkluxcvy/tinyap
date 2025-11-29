use crate::state::AppState;

use axum::{extract::FromRequestParts, http::request::Parts, response::Redirect};
use axum_extra::extract::cookie::CookieJar;

pub struct AuthUser {
    pub id: i64,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Redirect;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        let Some(cookie) = jar.get("session_id") else {
            return Err(Redirect::to("/login"));
        };

        let session_id = cookie.value();

        let row = sqlx::query!(
            "SELECT user_id FROM sessions
            WHERE session_id = ?
            AND expires_at > strftime('%s','now')",
            session_id
        )
        .fetch_optional(&state.db_pool)
        .await
        .unwrap();

        let Some(row) = row else {
            return Err(Redirect::to("/login"));
        };

        Ok(AuthUser { id: row.user_id })
    }
}

// return user id without authentication
pub struct MaybeAuthUser {
    pub id: Option<i64>,
}
impl FromRequestParts<AppState> for MaybeAuthUser {
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        let Some(cookie) = jar.get("session_id") else {
            return Ok(MaybeAuthUser { id: None });
        };

        let session_id = cookie.value();

        let row = sqlx::query!(
            "SELECT user_id FROM sessions
            WHERE session_id = ?
            AND expires_at > strftime('%s','now')",
            session_id
        )
        .fetch_optional(&state.db_pool)
        .await
        .unwrap();

        let Some(row) = row else {
            return Ok(MaybeAuthUser { id: None });
        };

        Ok(MaybeAuthUser {
            id: Some(row.user_id),
        })
    }
}

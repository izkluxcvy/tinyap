use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

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
            return Err(Redirect::to("/?message=login_required"));
        };

        let session_id = cookie.value();
        let session = queries::session::get(state, session_id, &utils::date_now()).await;
        let Some(session) = session else {
            return Err(Redirect::to("/?message=login_required"));
        };

        Ok(AuthUser {
            id: session.user_id,
        })
    }
}

use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    extract::{Form, State},
};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
pub struct TokenRequestForm {
    pub grant_type: String,
    pub code: String,
    pub client_id: i64,
    pub client_secret: String,
}

pub async fn post(State(state): State<AppState>, Form(req): Form<TokenRequestForm>) -> Json<Value> {
    // Check grant_type
    if req.grant_type != "authorization_code" {
        return Json(json!({"error": "invalid grant_type"}));
    }

    // Get app
    let Some(app) = queries::oauth::get_app(&state, req.client_id).await else {
        return Json(json!({"error": "invalid client"}));
    };

    // Check client_secret
    if app.client_secret != req.client_secret {
        return Json(json!({"error": "invalid client"}));
    }

    // Get authorization
    let Some(auth) = queries::oauth::get_authorization(&state, req.client_id, &req.code).await
    else {
        return Json(json!({"error": "invalid code"}));
    };

    // Generate token
    let token = utils::gen_secure_token();
    let expires_at = utils::date_plus_days(state.config.token_ttl_days);

    // Save token
    queries::oauth::create_token(&state, auth.user_id, req.client_id, &token, &expires_at).await;

    Json(json!({
        "access_token": token,
        "token_type": "Bearer",
        "scope": "read write",
        "expires_at": expires_at,
    }))
}

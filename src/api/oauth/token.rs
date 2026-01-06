use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    body::Body,
    extract::{FromRequest, Multipart, State},
    http::{Request, header::HeaderMap},
};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
pub struct TokenRequestRequest {
    pub grant_type: String,
    pub code: String,
    pub client_id: i64,
    pub client_secret: String,
}

pub async fn post(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: Request<Body>,
) -> Json<Value> {
    // Extract from either form or multipart
    let (grant_type, code, client_id, client_secret) =
        if let Some(content_type) = headers.get("content-type") {
            let content_type = content_type.to_str().unwrap_or("");
            if content_type.contains("multipart") {
                // Multipart
                let mut multipart = Multipart::from_request(req, &state).await.unwrap();
                let mut grant_type = String::new();
                let mut code = String::new();
                let mut client_id = 0;
                let mut client_secret = String::new();
                while let Some(field) = multipart.next_field().await.unwrap() {
                    let name = field.name().unwrap_or("").to_string();
                    let value = field.text().await.unwrap_or("".to_string());
                    match name.as_str() {
                        "grant_type" => grant_type = value,
                        "code" => code = value,
                        "client_id" => client_id = value.parse::<i64>().unwrap_or(0),
                        "client_secret" => client_secret = value,
                        _ => {}
                    }
                }

                (grant_type, code, client_id, client_secret)
            } else {
                // Form
                let Ok(body_bytes) =
                    axum::body::to_bytes(req.into_body(), state.config.max_note_chars).await
                else {
                    return Json(json!({"error": "failed to read body"}));
                };

                let Ok(req_form) = serde_urlencoded::from_bytes::<TokenRequestRequest>(&body_bytes)
                else {
                    return Json(json!({"error": "invalid form"}));
                };

                (
                    req_form.grant_type,
                    req_form.code,
                    req_form.client_id,
                    req_form.client_secret,
                )
            }
        } else {
            return Json(json!({"error": "missing content-type"}));
        };

    // Check grant_type
    if grant_type != "authorization_code" {
        return Json(json!({"error": "invalid grant_type"}));
    }

    // Get app
    let Some(app) = queries::oauth::get_app(&state, client_id).await else {
        return Json(json!({"error": "invalid client"}));
    };

    // Check client_secret
    if app.client_secret != client_secret {
        return Json(json!({"error": "invalid client"}));
    }

    // Get authorization
    let Some(auth) = queries::oauth::get_authorization(&state, client_id, &code).await else {
        return Json(json!({"error": "invalid code"}));
    };

    // Generate token
    let token = utils::gen_secure_token();
    let expires_at = utils::date_plus_days(state.config.token_ttl_days);

    // Save token
    queries::oauth::create_token(&state, auth.user_id, client_id, &token, &expires_at).await;

    // Delete expired tokens
    let date_now = utils::date_now();
    queries::oauth::delete_expired_tokens(&state, &date_now).await;

    Json(json!({
        "access_token": token,
        "token_type": "Bearer",
        "scope": "read write",
        "expires_at": expires_at,
    }))
}

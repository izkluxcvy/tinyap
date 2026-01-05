use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{Request, header::CONTENT_TYPE},
};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
pub struct AppRequest {
    pub client_name: String,
    pub redirect_uris: String,
}

pub async fn post(State(state): State<AppState>, req: Request<Body>) -> Json<Value> {
    // Extract from either form or json
    let (parts, body) = req.into_parts();
    let Ok(body_bytes) = axum::body::to_bytes(body, state.config.max_note_chars).await else {
        return Json(json!({"error": "failed to read body"}));
    };
    let (client_name, redirect_uris) = if let Some(content_type) = parts.headers.get(CONTENT_TYPE) {
        if content_type.to_str().unwrap_or("").contains("json") {
            let Ok(req_json) = serde_json::from_slice::<AppRequest>(&body_bytes) else {
                return Json(json!({"error": "invalid json"}));
            };

            (req_json.client_name, req_json.redirect_uris)
        } else {
            let Ok(req_form) = serde_urlencoded::from_bytes::<AppRequest>(&body_bytes) else {
                return Json(json!({"error": "invalid form data"}));
            };

            (req_form.client_name, req_form.redirect_uris)
        }
    } else {
        return Json(json!({"error": "missing content type"}));
    };

    // Create app
    let client_id = utils::gen_unique_id();
    let client_secret = utils::gen_secure_token();
    queries::oauth::create_app(
        &state,
        &client_name,
        &redirect_uris,
        client_id,
        &client_secret,
    )
    .await;

    Json(json!({
        "id": utils::gen_unique_id(),
        "redirect_uri": redirect_uris,
        "client_id": client_id.to_string(),
        "client_secret": client_secret,
    }))
}

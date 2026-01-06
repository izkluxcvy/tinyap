use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    body::Body,
    extract::{FromRequest, Multipart, State},
    http::{
        Request,
        header::{CONTENT_TYPE, HeaderMap},
    },
};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
pub struct AppRequest {
    pub client_name: String,
    pub redirect_uris: String,
}

pub async fn post(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: Request<Body>,
) -> Json<Value> {
    // Extract from either form, multipart, or json
    let (client_name, redirect_uris) = if let Some(content_type) = headers.get(CONTENT_TYPE) {
        let content_type = content_type.to_str().unwrap_or("");

        if content_type.contains("json") {
            // JSON
            let Ok(body_bytes) =
                axum::body::to_bytes(req.into_body(), state.config.max_note_chars).await
            else {
                return Json(json!({"error": "failed to read body"}));
            };

            let Ok(req_json) = serde_json::from_slice::<AppRequest>(&body_bytes) else {
                return Json(json!({"error": "invalid json"}));
            };

            (req_json.client_name, req_json.redirect_uris)
        } else if content_type.contains("multipart") {
            // Multipart
            let mut multipart = Multipart::from_request(req, &state).await.unwrap();
            let mut client_name = String::new();
            let mut redirect_uris = String::new();
            while let Some(field) = multipart.next_field().await.unwrap() {
                let name = field.name().unwrap_or("").to_string();
                let value = field.text().await.unwrap_or("".to_string());
                match name.as_str() {
                    "client_name" => client_name = value,
                    "redirect_uris" => redirect_uris = value,
                    _ => {}
                }
            }

            (client_name, redirect_uris)
        } else {
            // Form
            let Ok(body_bytes) =
                axum::body::to_bytes(req.into_body(), state.config.max_note_chars).await
            else {
                return Json(json!({"error": "failed to read body"}));
            };

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
        "name": client_name,
        "redirect_uri": redirect_uris,
        "client_id": client_id.to_string(),
        "client_secret": client_secret,
    }))
}

use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use axum::{
    Json,
    extract::{Form, State},
};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
pub struct AppRequestForm {
    pub client_name: String,
    pub redirect_uris: String,
}

pub async fn post(State(state): State<AppState>, Form(req): Form<AppRequestForm>) -> Json<Value> {
    let client_id = utils::gen_unique_id();
    let client_secret = utils::gen_secure_token();
    queries::oauth::create_app(
        &state,
        &req.client_name,
        &req.redirect_uris,
        client_id,
        &client_secret,
    )
    .await;

    Json(json!({
        "id": utils::gen_unique_id(),
        "redirect_uri": req.redirect_uris,
        "client_id": client_id,
        "client_secret": client_secret,
    }))
}

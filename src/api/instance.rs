use crate::VERSION;
use crate::back::init::AppState;

use axum::{Json, extract::State};
use serde_json::{Value, json};

pub async fn get_v1(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "uri": &state.domain,
        "title": &state.metadata.instance_name,
        "short_description": "",
        "description": &state.metadata.instance_description,
        "email": "",
        "version": VERSION,
    }))
}

pub async fn get_v2(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "domain": &state.domain,
        "title": &state.metadata.instance_name,
        "description": &state.metadata.instance_description,
        "version": VERSION,
    }))
}

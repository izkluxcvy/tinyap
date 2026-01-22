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
        "configuration": {
            "statuses": {
                "max_characters": state.config.max_note_chars,
                "max_media_attachments": 0,
            }
        },
    }))
}

pub async fn get_v2(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "domain": &state.domain,
        "title": &state.metadata.instance_name,
        "description": &state.metadata.instance_description,
        "version": VERSION,
        "registrations": {
            "enabled": false,
        },
        "configuration": {
            "statuses": {
                "max_characters": state.config.max_note_chars,
                "max_media_attachments": 0,
            }
        },
        "thumbnail": {},
        "contact": {
            "email": "",
        },
    }))
}

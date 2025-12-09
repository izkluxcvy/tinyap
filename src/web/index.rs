use crate::state::AppState;

use axum::{
    extract::{Query, State},
    response::Html,
};

#[derive(serde::Deserialize)]
pub struct MessageParam {
    pub message: Option<String>,
}

pub async fn page(
    State(state): State<AppState>,
    Query(MessageParam { message }): Query<MessageParam>,
) -> Html<String> {
    let message_body: &str;
    match message {
        Some(v) => {
            if v == "password_updated" {
                message_body = "Your password has been updated successfully."
            } else if v == "invalid_input" {
                message_body = "Invalid input. Please try again."
            } else if v == "username_taken" {
                message_body = "This username is already taken. Please choose another one."
            } else if v == "exceed_max_note_length" {
                message_body = "Your note exceeds the maximum allowed length."
            } else {
                message_body = "";
            }
        }
        None => {
            message_body = "";
        }
    }
    let mut context = tera::Context::new();
    context.insert("site_name", &state.site_name);
    context.insert("message", message_body);
    let rendered = state.tera.render("index.html", &context).unwrap();
    Html(rendered)
}

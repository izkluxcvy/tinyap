use crate::back::init::AppState;

use axum::{
    extract::{Query, State},
    response::Html,
};

#[derive(serde::Deserialize)]
pub struct MessageQuery {
    pub message: Option<String>,
}

pub async fn get(State(state): State<AppState>, Query(query): Query<MessageQuery>) -> Html<String> {
    let message_body = if let Some(msg) = query.message {
        match msg.as_str() {
            "login_required" => "You need to be logged in first.",
            _ => "",
        }
    } else {
        ""
    };

    let mut context = tera::Context::new();
    context.insert("message", &message_body);
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("instance_description", &state.metadata.instance_description);
    let rendered = state.tera.render("index.html", &context).unwrap();

    Html(rendered)
}

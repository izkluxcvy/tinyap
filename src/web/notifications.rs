use crate::back::init::AppState;
use crate::back::queries;
use crate::web::auth::AuthUser;

use axum::{
    extract::State,
    response::{Html, IntoResponse},
};

pub async fn get(State(state): State<AppState>, user: AuthUser) -> impl IntoResponse {
    // Get user
    let user = queries::user::get_by_id(&state, user.id).await;

    // Get notifications
    let notifications =
        queries::notification::get(&state, user.id, state.web_config.max_timeline_items).await;

    // Rendering
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    context.insert("timezone", &state.web_config.timezone);
    context.insert("username", &user.username);
    context.insert("notifications", &notifications);
    let rendered = state.tera.render("notifications.html", &context).unwrap();

    Html(rendered).into_response()
}

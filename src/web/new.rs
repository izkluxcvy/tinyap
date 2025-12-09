use crate::{auth::AuthUser, state::AppState};

use axum::{
    extract::{Query, State},
    response::Html,
};

#[derive(serde::Deserialize)]
pub struct ReplyParam {
    pub in_reply_to: Option<String>,
}

pub async fn page(
    _user: AuthUser,
    State(state): State<AppState>,
    Query(ReplyParam { in_reply_to }): Query<ReplyParam>,
) -> Html<String> {
    let mut is_reply = false;
    if in_reply_to.is_some() {
        is_reply = true;
    };

    let mut context = tera::Context::new();
    context.insert("site_name", &state.site_name);
    context.insert("is_reply", &is_reply);
    context.insert("in_reply_to", &in_reply_to);
    let rendered = state.tera.render("new.html", &context).unwrap();
    Html(rendered)
}

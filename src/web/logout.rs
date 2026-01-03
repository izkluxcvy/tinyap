use crate::back::init::AppState;
use crate::back::queries;
use crate::web::auth::AuthUser;

use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};

pub async fn get(State(state): State<AppState>, _user: AuthUser) -> Html<String> {
    let mut context = tera::Context::new();
    context.insert("instance_name", &state.metadata.instance_name);
    let rendered = state.tera.render("logout.html", &context).unwrap();

    Html(rendered)
}

pub async fn post(
    State(state): State<AppState>,
    jar: CookieJar,
    _user: AuthUser,
) -> impl IntoResponse {
    let Some(cookie) = jar.get("session_id") else {
        return Redirect::to("/").into_response();
    };
    let session_id = cookie.value();

    queries::session::delete_by_session_id(&state, session_id).await;

    let cookie = Cookie::build(("session_id", ""))
        .path("/")
        .http_only(true)
        .secure(true)
        .max_age(time::Duration::seconds(0));

    (jar.add(cookie), Redirect::to("/")).into_response()
}

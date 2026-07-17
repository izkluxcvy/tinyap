use crate::back::init::AppState;
use crate::back::queries;
use crate::back::user;
use crate::back::utils;

use axum::{
    extract::{Form, Query, State},
    response::{Html, IntoResponse, Redirect},
};

#[derive(serde::Deserialize)]
pub struct AuthorizeQuery {
    pub response_type: String,
    pub client_id: i64,
    pub redirect_uri: String,
    pub state: Option<String>,
}

pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<AuthorizeQuery>,
) -> impl IntoResponse {
    // Check response_type
    if query.response_type != "code" {
        return "Invalid response_type".into_response();
    }

    // Get app
    let Some(app) = queries::oauth::get_app(&state, query.client_id).await else {
        return "Invalid client_id".into_response();
    };

    // Check redirect_uri
    if app.redirect_uri != query.redirect_uri {
        return "Mismatched redirect_uri".into_response();
    }

    // Render
    Html(format!(
        r#"<h3>Authorize for {}</h3>
        <form action="/oauth/authorize" method="post">
        <label for="username">Username:</label><br>
        <input type="text" id="username" name="username"><br>
        <label for="password">Password:</label><br>
        <input type="password" id="password" name="password"><br>
        <input type="hidden" name="client_id" value="{}">
        <input type="hidden" name="redirect_uri" value="{}">
        <input type="hidden" name="state" value="{}">
        <button type="submit">Authorize</button>
        </form>"#,
        tera::escape_html(&app.app_name),
        query.client_id,
        tera::escape_html(&query.redirect_uri),
        tera::escape_html(query.state.as_deref().unwrap_or(""))
    ))
    .into_response()
}

#[derive(serde::Deserialize)]
pub struct AuthorizeForm {
    pub username: String,
    pub password: String,
    pub client_id: i64,
    pub redirect_uri: String,
    pub state: String,
}

pub async fn post(
    State(state): State<AppState>,
    Form(form): Form<AuthorizeForm>,
) -> impl IntoResponse {
    let Ok(user_id) = user::verify_password(&state, &form.username, &form.password).await else {
        return "Invalid username or password".into_response();
    };

    // Check app
    let Some(app) = queries::oauth::get_app(&state, form.client_id).await else {
        return "Invalid client_id".into_response();
    };
    if app.redirect_uri != form.redirect_uri {
        return "Mismatched redirect_uri".into_response();
    }

    // Generate authorization code
    let code = utils::gen_secure_token();

    // Save authorization
    queries::oauth::create_authorization(&state, user_id, form.client_id, &code).await;

    if form.redirect_uri == "urn:ietf:wg:oauth:2.0:oob" {
        Html(format!(r#"<p>Your authorization code is: {}</p>"#, code)).into_response()
    } else {
        if !form.state.is_empty() {
            return Redirect::to(&format!(
                "{}?code={}&state={}",
                form.redirect_uri, code, &form.state
            ))
            .into_response();
        }
        Redirect::to(&format!("{}?code={}", form.redirect_uri, code)).into_response()
    }
}

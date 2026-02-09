use crate::back::init::AppState;

use sqlx::{query, query_as};

pub async fn create_app(
    state: &AppState,
    app_name: &str,
    redirect_uri: &str,
    client_id: i64,
    client_secret: &str,
) {
    query(
        "INSERT INTO oauth_apps (app_name, redirect_uri, client_id, client_secret)
        VALUES ($1, $2, $3, $4)",
    )
    .bind(app_name)
    .bind(redirect_uri)
    .bind(client_id)
    .bind(client_secret)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

#[derive(sqlx::FromRow)]
pub struct AppRecord {
    pub app_name: String,
    pub client_secret: String,
}
pub async fn get_app(state: &AppState, client_id: i64) -> Option<AppRecord> {
    query_as("SELECT app_name, client_secret FROM oauth_apps WHERE client_id = $1")
        .bind(client_id)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap()
}

#[cfg(feature = "web")]
pub async fn delete_app(state: &AppState, client_id: i64, user_id: i64) {
    query(
        "DELETE FROM oauth_apps
        WHERE client_id = $1
        AND client_id IN (SELECT client_id FROM oauth_tokens WHERE user_id = $2)",
    )
    .bind(client_id)
    .bind(user_id)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

pub async fn create_authorization(state: &AppState, user_id: i64, client_id: i64, code: &str) {
    query(
        "INSERT INTO oauth_authorizations (user_id, client_id, code)
        VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(client_id)
    .bind(code)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

#[derive(sqlx::FromRow)]
pub struct AuthorizationRecord {
    pub user_id: i64,
}

pub async fn get_authorization(
    state: &AppState,
    client_id: i64,
    code: &str,
) -> Option<AuthorizationRecord> {
    query_as(
        "SELECT user_id FROM oauth_authorizations
        WHERE client_id = $1 AND code = $2",
    )
    .bind(client_id)
    .bind(code)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap()
}

pub async fn create_token(
    state: &AppState,
    user_id: i64,
    client_id: i64,
    token: &str,
    expires_at: &str,
) {
    query(
        "INSERT INTO oauth_tokens (user_id, client_id, token, expires_at)
        VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(client_id)
    .bind(token)
    .bind(expires_at)
    .execute(&state.db_pool)
    .await
    .unwrap();
}

#[derive(sqlx::FromRow)]
pub struct TokenRecord {
    pub user_id: i64,
}

pub async fn get_token(state: &AppState, token: &str, date_now: &str) -> Option<TokenRecord> {
    query_as("SELECT user_id FROM oauth_tokens WHERE token = $1 AND expires_at > $2")
        .bind(token)
        .bind(date_now)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap()
}

#[cfg(feature = "web")]
#[derive(sqlx::FromRow, serde::Serialize)]
pub struct TokenWithAppRecord {
    pub client_id: i64,
    pub app_name: String,
    pub expires_at: String,
}
#[cfg(feature = "web")]
pub async fn get_tokens(state: &AppState, user_id: i64) -> Vec<TokenWithAppRecord> {
    query_as(
        "SELECT a.client_id, a.app_name, t.expires_at
        FROM oauth_tokens as t
        JOIN oauth_apps as a ON t.client_id = a.client_id
        WHERE t.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap()
}

pub async fn delete_expired_tokens(state: &AppState, date_now: &str) {
    query("DELETE FROM oauth_tokens WHERE expires_at <= $1")
        .bind(date_now)
        .execute(&state.db_pool)
        .await
        .unwrap();
}

pub async fn delete_unused_apps(state: &AppState) {
    query(
        "DELETE FROM oauth_apps
        WHERE client_id NOT IN (SELECT DISTINCT client_id FROM oauth_tokens)",
    )
    .execute(&state.db_pool)
    .await
    .unwrap();
}

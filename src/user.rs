use crate::ap::utils;
use crate::auth::AuthUser;
use crate::state::AppState;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1::EncodeRsaPrivateKey, pkcs1::EncodeRsaPublicKey};
use time::{Duration, OffsetDateTime};
use url::Url;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SignupForm {
    pub username: String,
    pub password: String,
}

pub async fn signup(
    State(state): State<AppState>,
    Form(form): Form<SignupForm>,
) -> impl IntoResponse {
    // Validation
    if state.config.allow_signup == false {
        return Redirect::to("/");
    }
    if form.username.is_empty() || form.password.is_empty() {
        return Redirect::to("/?message=invalid_input");
    }
    if form.username.len() > 32 {
        return Redirect::to("/?message=invalid_input");
    }
    if !form.username.bytes().all(|a| u8::is_ascii_alphanumeric(&a)) {
        return Redirect::to("/?message=invalid_input");
    }

    let existing_user = sqlx::query!("SELECT id FROM users WHERE username = ?", form.username)
        .fetch_optional(&state.db_pool)
        .await
        .expect("Failed to query database");
    if existing_user.is_some() {
        return Redirect::to("/?message=username_taken");
    }

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(form.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let mut rng = rand::rngs::OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
    let public_key = RsaPublicKey::from(&private_key);

    let private_pem = private_key
        .to_pkcs1_pem(Default::default())
        .unwrap()
        .to_string();
    let public_pem = public_key.to_pkcs1_pem(Default::default()).unwrap();

    let actor_id = format!("https://{}/users/{}", state.domain, form.username);

    let created_at = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    sqlx::query!(
        "INSERT INTO users (username, password_hash, actor_id, private_key, public_key, display_name, bio, created_at, is_local)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        form.username,
        password_hash,
        actor_id,
        private_pem,
        public_pem,
        form.username,
        "",
        created_at,
        1
    )
    .execute(&state.db_pool)
    .await
    .expect("Failed to insert user into database");

    Redirect::to("/login")
}

pub async fn create_remoteuser(actor_url: &str, state: &AppState) {
    let existing_user = sqlx::query!("SELECT id FROM users WHERE actor_id = ?", actor_url)
        .fetch_optional(&state.db_pool)
        .await
        .expect("Failed to query database");
    if existing_user.is_some() {
        return;
    }

    let actor_json = {
        let client = reqwest::Client::new();
        let res = client
            .get(actor_url)
            .header("Accept", "application/activity+json")
            .send()
            .await
            .expect("Failed to fetch remote actor");
        res.json::<serde_json::Value>()
            .await
            .expect("Failed to parse remote actor JSON")
    };

    let url = Url::parse(actor_url).expect("Invalid actor URL");
    let host = url.host_str().unwrap();
    let username = actor_json["preferredUsername"]
        .as_str()
        .unwrap()
        .to_string();
    let name_and_host = format!("{}@{}", username, host);
    let display_name = actor_json["name"].as_str().unwrap_or(&username).to_string();
    let bio = actor_json["summary"].as_str().unwrap_or("").to_string();
    let bio_clean = utils::strip_html_tags(&bio);
    let created_at = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();

    sqlx::query!(
        "INSERT INTO users (username, actor_id, display_name, bio, created_at, is_local)
        VALUES (?, ?, ?, ?, ?, ?)",
        name_and_host,
        actor_url,
        display_name,
        bio_clean,
        created_at,
        0,
    )
    .execute(&state.db_pool)
    .await
    .expect("Failed to insert remote user into database");
}

pub async fn update_remoteuser(actor_url: &str, state: &AppState) {
    let actor_json = {
        let client = reqwest::Client::new();
        let res = client
            .get(actor_url)
            .header("Accept", "application/activity+json")
            .send()
            .await
            .expect("Failed to fetch remote actor");
        res.json::<serde_json::Value>()
            .await
            .expect("Failed to parse remote actor JSON")
    };

    let username = actor_json["preferredUsername"]
        .as_str()
        .unwrap()
        .to_string();
    let display_name = actor_json["name"].as_str().unwrap_or(&username).to_string();
    let bio = actor_json["summary"].as_str().unwrap_or("").to_string();
    let bio_clean = utils::strip_html_tags(&bio);

    sqlx::query!(
        "UPDATE users
        SET display_name = ?, bio = ?
        WHERE actor_id = ?",
        display_name,
        bio_clean,
        actor_url
    )
    .execute(&state.db_pool)
    .await
    .expect("Failed to update remote user in database");
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let row = sqlx::query!(
        "SELECT id, password_hash, is_local FROM users WHERE username = ?",
        form.username
    )
    .fetch_optional(&state.db_pool)
    .await
    .expect("Failed to fetch user from database");

    let Some(row) = row else {
        return Redirect::to("/login").into_response();
    };

    if &row.is_local == &0 {
        return Redirect::to("/login").into_response();
    }
    let password_hash = &row.password_hash.unwrap();
    let parsed_hash = PasswordHash::new(&password_hash).unwrap();
    let argon2 = Argon2::default();
    if argon2
        .verify_password(form.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Redirect::to("/login").into_response();
    }

    let session_id = Uuid::new_v4().to_string();
    let expires_at = (OffsetDateTime::now_utc() + Duration::days(state.config.session_ttl_days))
        .unix_timestamp();

    sqlx::query!(
        "INSERT INTO sessions (session_id, user_id, expires_at) VALUES (?, ?, ?)",
        session_id,
        row.id,
        expires_at
    )
    .execute(&state.db_pool)
    .await
    .expect("Failed to create session");

    // Delete old sessions
    let old_sessions = sqlx::query!(
        "SELECT session_id FROM sessions
        WHERE user_id = $1
        ORDER BY expires_at ASC
        LIMIT (SELECT MAX((SELECT COUNT(*) - $2 FROM sessions WHERE user_id = $1), 0))",
        row.id,
        state.config.session_max_per_user
    )
    .fetch_all(&state.db_pool)
    .await
    .expect("Failed to fetch old sessions");

    for session in old_sessions {
        sqlx::query!(
            "DELETE FROM sessions WHERE session_id = ?",
            session.session_id
        )
        .execute(&state.db_pool)
        .await
        .expect("Failed to delete old session");
    }

    let cookie = Cookie::build(("session_id", session_id))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(120));

    (jar.add(cookie), Redirect::to("/home")).into_response()
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    user: AuthUser,
) -> impl IntoResponse {
    let Some(cookie) = jar.get("session_id") else {
        return Redirect::to("/login").into_response();
    };

    let session_id = cookie.value();

    sqlx::query!(
        "DELETE FROM sessions WHERE session_id = ? AND user_id = ?",
        session_id,
        user.id
    )
    .execute(&state.db_pool)
    .await
    .expect("Failed to delete session");

    let cookie = Cookie::build(("session_id", ""))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::seconds(0));

    (jar.add(cookie), Redirect::to("/login")).into_response()
}

#[derive(serde::Deserialize)]
pub struct UpdateProfileForm {
    pub display_name: String,
    pub bio: String,
    pub username: String,
}

pub async fn update_profile(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<UpdateProfileForm>,
) -> impl IntoResponse {
    sqlx::query!(
        "UPDATE users
        SET display_name = ?, bio = ?
        WHERE id = ?",
        form.display_name,
        form.bio,
        user.id
    )
    .execute(&state.db_pool)
    .await
    .expect("Failed to update profile");

    Redirect::to(&format!("/@{}", form.username))
}

#[derive(serde::Deserialize)]
pub struct UpdatePasswordForm {
    pub new_password: String,
}

pub async fn update_password(
    State(state): State<AppState>,
    user: AuthUser,
    Form(form): Form<UpdatePasswordForm>,
) -> impl IntoResponse {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(form.new_password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    sqlx::query!(
        "UPDATE users
        SET password_hash = ?
        WHERE id = ?",
        password_hash,
        user.id
    )
    .execute(&state.db_pool)
    .await
    .expect("Failed to update password");

    Redirect::to("/?message=password_updated")
}

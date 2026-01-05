use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs8::EncodePrivateKey, pkcs8::EncodePublicKey};
use serde_json::Value;
use url::Url;

pub async fn add(state: &AppState, username: &str, password: &str) -> Result<(), String> {
    // Validation
    if username.is_empty() || password.is_empty() {
        return Err("Username and password cannot be empty".to_string());
    }

    if username.len() > 32 {
        return Err("Username is too long (max 32 characters)".to_string());
    }

    if !username.bytes().all(|a| u8::is_ascii_alphanumeric(&a)) {
        return Err("Username can only contain alphanumeric characters".to_string());
    }

    let existing = queries::user::get_by_username(state, username).await;
    if existing.is_some() {
        return Err("Username already exists".to_string());
    }

    // Hash password
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    // Generate RSA key pair
    let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
    let public_key = RsaPublicKey::from(&private_key);

    let private_key_pem = private_key
        .to_pkcs8_pem(Default::default())
        .unwrap()
        .to_string();
    let public_key_pem = public_key.to_public_key_pem(Default::default()).unwrap();

    // Create user
    let ap_url = utils::local_user_ap_url(&state.domain, username);
    let inbox_url = utils::local_user_inbox_url(&state.domain, username);
    let created_at = utils::date_now();

    queries::user::create(
        state,
        username,
        Some(&password_hash),
        &ap_url,
        &inbox_url,
        Some(&private_key_pem),
        Some(&public_key_pem),
        username,
        "",
        &created_at,
        &created_at,
        1,
    )
    .await;

    Ok(())
}

#[cfg(feature = "web")]
pub async fn update_profile(state: &AppState, user_id: i64, display_name: &str, bio: &str) {
    let bio = utils::parse_content(state, bio);
    queries::user::update_profile(state, user_id, display_name, &bio).await;
}

pub async fn update_password(state: &AppState, user_id: i64, password: &str) {
    // Hash password
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    queries::user::update_password(state, user_id, &password_hash).await;
}

pub async fn verify_password(state: &AppState, username: &str, password: &str) -> Result<i64, ()> {
    let user = queries::user::get_by_username(state, username).await;
    // Check user exists
    let Some(user) = user else {
        return Err(());
    };

    // Check password hash exists (is local)
    let Some(stored_hash) = user.password_hash else {
        return Err(());
    };

    // Verify password
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&stored_hash).unwrap();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(user.id),
        Err(_) => Err(()),
    }
}

pub async fn fetch_remote(
    state: &AppState,
    ap_url: &str,
) -> Result<(String, String, String, String, String), String> {
    // Fetch
    let Ok(res) = utils::signed_get(state, ap_url).await else {
        return Err("Failed to fetch remote user".to_string());
    };

    // Validation
    let Ok(user_json) = res.json::<Value>().await else {
        return Err("Fetched object is not valid JSON".to_string());
    };

    if user_json["type"] != "Person" {
        return Err("Fetched object is not a Person".to_string());
    }

    let Some(username) = user_json["preferredUsername"].as_str() else {
        return Err("Fetched object does not have a preferredUsername".to_string());
    };

    let Some(ap_url) = user_json["id"].as_str() else {
        return Err("Fetched object does not have an id".to_string());
    };

    let Some(inbox_url) = user_json["inbox"].as_str() else {
        return Err("Fetched object does not have an inbox".to_string());
    };

    let display_name = user_json["name"].as_str().unwrap_or(username);
    // Merge attachments to bio
    let mut bio = user_json["summary"].as_str().unwrap_or("").to_string();
    if let Some(attachments) = user_json["attachment"].as_array() {
        bio.push_str("\n");
        for attachment in attachments {
            if let (Some(name), Some(url)) =
                (attachment["name"].as_str(), attachment["value"].as_str())
            {
                bio.push_str(&format!("\n{}: {}", name, url));
            }
        }
    }
    let bio = utils::strip_content(state, &bio);
    let bio = utils::parse_content(state, &bio);

    let url = Url::parse(ap_url).unwrap();
    let host = url.host_str().unwrap();

    let username = format!("{}@{}", username, host);

    Ok((
        username,
        ap_url.to_string(),
        inbox_url.to_string(),
        display_name.to_string(),
        bio,
    ))
}

pub async fn add_remote(state: &AppState, ap_url: &str) -> Result<(), String> {
    let Ok((username, ap_url, inbox_url, display_name, bio)) = fetch_remote(state, ap_url).await
    else {
        return Err("Failed to fetch remote user".to_string());
    };

    queries::user::create(
        state,
        &username,
        None,
        &ap_url,
        &inbox_url,
        None,
        None,
        &display_name,
        &bio,
        &utils::date_now(),
        &utils::date_now(),
        0,
    )
    .await;

    Ok(())
}

pub async fn update_remote(state: &AppState, ap_url: &str) -> Result<(), String> {
    let Ok((_username, ap_url, _inbox_url, display_name, bio)) = fetch_remote(state, ap_url).await
    else {
        return Err("Failed to fetch remote user".to_string());
    };

    let Some(user) = queries::user::get_by_ap_url(state, &ap_url).await else {
        return Err("User not found".to_string());
    };

    queries::user::update_profile(state, user.id, &display_name, &bio).await;

    Ok(())
}

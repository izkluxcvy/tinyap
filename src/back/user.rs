use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1::EncodeRsaPrivateKey, pkcs1::EncodeRsaPublicKey};

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
        .to_pkcs1_pem(Default::default())
        .unwrap()
        .to_string();
    let public_key_pem = public_key.to_pkcs1_pem(Default::default()).unwrap();

    // Create user
    let ap_url = utils::local_user_ap_url(&state.domain, username);
    let inbox_url = utils::local_user_inbox_url(&state.domain, username);
    let created_at = utils::date_now();

    queries::user::create(
        state,
        username,
        &password_hash,
        &ap_url,
        &inbox_url,
        &private_key_pem,
        &public_key_pem,
        username,
        "",
        &created_at,
        &created_at,
        1,
    )
    .await;

    Ok(())
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

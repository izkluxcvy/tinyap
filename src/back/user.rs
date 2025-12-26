use crate::back::init::AppState;
use crate::back::queries;
use crate::back::utils;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1::EncodeRsaPrivateKey, pkcs1::EncodeRsaPublicKey};

pub async fn add_user(state: &AppState, username: &str, password: &str) {
    let existing = queries::get_userid_by_username(state, username).await;
    if existing.is_some() {
        println!("username {} already exists", username);
        return;
    }

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
    let public_key = RsaPublicKey::from(&private_key);

    let private_key_pem = private_key
        .to_pkcs1_pem(Default::default())
        .unwrap()
        .to_string();
    let public_key_pem = public_key.to_pkcs1_pem(Default::default()).unwrap();

    let ap_url = format!("https://{}/users/{}", state.domain, username);
    let inbox_url = format!("{}/inbox", ap_url);
    let created_at = utils::date_now();

    queries::create_user(
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
}

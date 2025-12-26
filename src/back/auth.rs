use crate::back::init::AppState;
use crate::back::queries;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};

pub async fn verify_user_password(
    state: &AppState,
    username: &str,
    password: &str,
) -> Result<i64, ()> {
    let user = queries::get_user_by_username(state, username).await;
    let Some(user) = user else {
        return Err(());
    };

    let Some(stored_hash) = user.password_hash else {
        return Err(());
    };

    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&stored_hash).unwrap();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(user.id),
        Err(_) => Err(()),
    }
}

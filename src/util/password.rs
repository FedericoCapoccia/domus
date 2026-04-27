use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use tokio::task::JoinError;

#[derive(Debug, thiserror::Error)]
pub enum PasswordHashError {
    #[error("failed to hash password")]
    Hash(String),
    #[error("password hash task failed")]
    Task(#[from] JoinError),
}

#[derive(Debug, thiserror::Error)]
pub enum PasswordVerifyError {
    #[error("failed to parse password hash")]
    Parse(String),
    #[error("password verification task failed")]
    Task(#[from] JoinError),
}

pub async fn hash(password: &str) -> Result<String, PasswordHashError> {
    let password = password.to_string();
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
    })
    .await
    .map_err(PasswordHashError::Task)?
    .map_err(|e| PasswordHashError::Hash(e.to_string()))
}

pub async fn verify(password: &str, stored_hash: &str) -> Result<bool, PasswordVerifyError> {
    let password = password.to_string();
    let stored_hash = stored_hash.to_string();

    tokio::task::spawn_blocking(move || {
        let hash = PasswordHash::new(&stored_hash)
            .map_err(|e| PasswordVerifyError::Parse(e.to_string()))?;
        match Argon2::default().verify_password(password.as_bytes(), &hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })
    .await
    .map_err(PasswordVerifyError::Task)?
}

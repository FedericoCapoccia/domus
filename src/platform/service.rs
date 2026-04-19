use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sqlx::PgPool;

use crate::platform::types::{LoginError, LoginRequest, User};

pub async fn login(pool: PgPool, req: LoginRequest) -> Result<String, LoginError> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, email, password_hash FROM platform_user WHERE email = $1",
        req.email
    )
    .fetch_optional(&pool)
    .await
    .map_err(LoginError::Database)?
    .ok_or(LoginError::UserNotFound(req.email))?;

    let pwd_hash =
        PasswordHash::new(&user.password_hash).map_err(|_| LoginError::PasswordParsing)?;

    Argon2::default()
        .verify_password(&req.password.into_bytes(), &pwd_hash)
        .map_err(LoginError::PasswordMismatch)?;

    Ok("MockJWT".to_string())
}

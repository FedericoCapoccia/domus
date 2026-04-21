use anyhow::anyhow;
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use sqlx::PgPool;

use crate::platform::types::{LoginError, LoginRequest, User};

pub async fn login(pool: PgPool, req: LoginRequest) -> Result<String, LoginError> {
    let user = sqlx::query_as!(
        User, // 'role as "role: _"' is needed because sqlx doesn't have knowledge about user defined types
        r#"SELECT id, email, password_hash, role as "role: _" FROM platform_user WHERE email = $1"#,
        req.email.trim().to_lowercase()
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

pub async fn ensure_owner(pool: PgPool) -> Result<(), anyhow::Error> {
    let exists =
        sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM platform_user WHERE role = 'owner')")
            .fetch_one(&pool)
            .await
            .map_err(|err| anyhow!("DB error: {err}"))?
            .unwrap_or(false);

    if exists {
        tracing::info!("Platform owner exists, skipping bootstrap");
        return Ok(());
    }

    let email = std::env::var("PLATFORM_OWNER_EMAIL")
        .map_err(|_| anyhow::anyhow!("Bootstrap failed, PLATFORM_OWNER_EMAIL not set"))?;
    let password = std::env::var("PLATFORM_OWNER_PASSWORD")
        .map_err(|_| anyhow::anyhow!("Bootstrap failed, PLATFORM_OWNER_PASSWORD not set"))?;

    if email.trim().is_empty() || password.is_empty() {
        return Err(anyhow!(
            "Bootstrap failed, PLATFORM_OWNER_EMAIL and PLATFORM_OWNER_PASSWORD must not be empty"
        ));
    }

    tracing::info!("No platform owner found, creating from environment");

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();

    sqlx::query!(
        "INSERT INTO platform_user (email, password_hash, role) VALUES ($1, $2, 'owner')",
        email.trim().to_lowercase(),
        password_hash,
    )
    .execute(&pool)
    .await
    .map_err(|err| anyhow!("Failed to create platform owner: {err}"))?;

    tracing::info!("Created platform owner: {email}");
    Ok(())
}

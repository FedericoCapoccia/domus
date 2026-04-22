use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use sqlx::PgPool;

use crate::platform::types::{
    LoginError, PlatformRole, UserCreateError, UserCreatedResponse, UserLoginInfo,
};

pub async fn login(pool: &PgPool, email: &str, password: &str) -> Result<String, LoginError> {
    let user = sqlx::query_as!(
        UserLoginInfo, // 'role as "role: _"' is needed because sqlx doesn't have knowledge about user defined types
        r#"SELECT id, email, password_hash, role as "role: _" FROM platform_user WHERE email = $1"#,
        email.trim().to_lowercase()
    )
    .fetch_optional(pool)
    .await
    .map_err(LoginError::Database)?
    .ok_or(LoginError::UserNotFound(email.to_string()))?;

    let pwd_hash =
        PasswordHash::new(&user.password_hash).map_err(|_| LoginError::PasswordParsing)?;

    Argon2::default()
        .verify_password(password.as_bytes(), &pwd_hash)
        .map_err(LoginError::PasswordMismatch)?;

    Ok("MockJWT".to_string())
}

pub async fn register_user(
    pool: &PgPool,
    email: &str,
    password: &str,
    role: PlatformRole,
) -> Result<UserCreatedResponse, UserCreateError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(UserCreateError::PasswordHashing)?
        .to_string();

    let result = sqlx::query_as!(
        UserCreatedResponse,
        r#"
        INSERT INTO platform_user (email, password_hash, role)
        VALUES ($1, $2, $3)
        RETURNING id, email, role as "role: _", created_at
        "#,
        email.trim().to_lowercase(),
        hash,
        role as PlatformRole,
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(user) => Ok(user),
        Err(sqlx::Error::Database(db_err)) => {
            if db_err.constraint() == Some("platform_user_email_unique") {
                Err(UserCreateError::EmailExists(email.to_string()))
            } else if db_err.constraint() == Some("platform_user_single_owner_idx") {
                Err(UserCreateError::OwnerExists(email.to_string()))
            } else {
                Err(UserCreateError::Database(sqlx::Error::Database(db_err)))
            }
        }
        Err(err) => Err(UserCreateError::Database(err)),
    }
}

pub async fn ensure_owner(pool: &PgPool) -> Result<(), anyhow::Error> {
    let exists =
        sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM platform_user WHERE role = 'owner')")
            .fetch_one(pool)
            .await
            .map_err(|err| anyhow::anyhow!("DB error: {err}"))?
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
        return Err(anyhow::anyhow!(
            "Bootstrap failed, PLATFORM_OWNER_EMAIL and PLATFORM_OWNER_PASSWORD must not be empty"
        ));
    }

    tracing::info!("No platform owner found, creating from environment");
    if let Err(error) = register_user(pool, &email, &password, PlatformRole::Owner).await {
        return Err(anyhow::anyhow!("Failed to create owner: {error}"));
    };

    tracing::info!("Created platform owner: {email}");
    Ok(())
}

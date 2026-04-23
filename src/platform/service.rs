use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::platform::{
    domain::PlatformRole,
    dto::UserCreatedResponse,
    error::{BootstrapError, LoginError, UserCreateError},
};

#[derive(sqlx::FromRow)]
pub struct UserLoginInfo {
    pub id: Uuid,
    pub password_hash: String,
    pub role: PlatformRole,
}

pub async fn login(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<UserLoginInfo, LoginError> {
    let user = sqlx::query_as!(
        UserLoginInfo, // 'role as "role: _"' is needed because sqlx doesn't have knowledge about user defined types
        r#"SELECT id, password_hash, role as "role: _" FROM platform_user WHERE email = $1"#,
        email
    )
    .fetch_optional(pool)
    .await?;

    let user = match user {
        Some(user) => user,
        None => {
            let _ = verify_password(
                password,
                "$argon2id$v=19$m=19456,t=2,p=1$UEViVXBNSThsbjJhSURLSg$o6V/wycFOBK3Th3a26vAwg",
            )
            .await;
            return Err(LoginError::UserNotFound(email.into()));
        }
    };

    verify_password(password, &user.password_hash).await?;
    Ok(user)
}

pub async fn register_user(
    pool: &PgPool,
    email: &str,
    password: &str,
    role: PlatformRole,
) -> Result<UserCreatedResponse, UserCreateError> {
    let hash = hash_password(password).await?;
    let result = sqlx::query_as!(
        UserCreatedResponse,
        r#"
        INSERT INTO platform_user (email, password_hash, role)
        VALUES ($1, $2, $3)
        RETURNING id, role as "role: _", created_at
        "#,
        email,
        hash,
        role as PlatformRole,
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(user) => Ok(user),
        Err(sqlx::Error::Database(db_err)) => {
            if db_err.constraint() == Some("platform_user_email_unique") {
                Err(UserCreateError::EmailExists(email.into()))
            } else if db_err.constraint() == Some("platform_user_single_owner_idx") {
                Err(UserCreateError::OwnerExists(email.into()))
            } else {
                Err(UserCreateError::Database(sqlx::Error::Database(db_err)))
            }
        }
        Err(err) => Err(UserCreateError::Database(err)),
    }
}

pub async fn ensure_owner(pool: &PgPool) -> Result<(), BootstrapError> {
    if owner_exists(pool).await? {
        tracing::info!("Platform owner exists, skipping bootstrap");
        return Ok(());
    }

    let email = std::env::var("PLATFORM_OWNER_EMAIL").map_err(|_| BootstrapError::MissingEmail)?;
    let password =
        std::env::var("PLATFORM_OWNER_PASSWORD").map_err(|_| BootstrapError::MissingPassword)?;

    if email.trim().is_empty() {
        return Err(BootstrapError::MissingEmail);
    }

    if password.len() < 8 || password.len() > 128 {
        return Err(BootstrapError::InvalidPasswordLength);
    }

    tracing::info!("No platform owner found, creating from environment");

    let email = email.trim().to_lowercase();
    match register_user(pool, &email, &password, PlatformRole::Owner).await {
        Ok(_) => {
            tracing::info!("Created platform owner");
            Ok(())
        }
        Err(UserCreateError::OwnerExists(_) | UserCreateError::EmailExists(_)) => {
            // This is more of a safeguard
            if owner_exists(pool).await? {
                tracing::warn!("Platform owner was created concurrently, continuing startup");
                Ok(())
            } else {
                Err(BootstrapError::CreateFailed)
            }
        }
        Err(_) => Err(BootstrapError::CreateFailed),
    }
}

async fn owner_exists(pool: &PgPool) -> Result<bool, sqlx::Error> {
    let res =
        sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM platform_user WHERE role = 'owner')")
            .fetch_one(pool)
            .await?
            .unwrap_or(false);
    Ok(res)
}

async fn hash_password(password: &str) -> Result<String, UserCreateError> {
    let password = password.to_string();

    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
    })
    .await
    .map_err(|e| UserCreateError::PasswordHashing(format!("Task panic: {e}")))?
    .map_err(|e| UserCreateError::PasswordHashing(format!("Argon2: {e}")))
}

async fn verify_password(password: &str, stored_hash: &str) -> Result<(), LoginError> {
    let password = password.to_string();
    let stored_hash = stored_hash.to_string();

    tokio::task::spawn_blocking(move || {
        let hash = PasswordHash::new(&stored_hash)
            .map_err(|e| LoginError::PasswordParse(format!("Argon2: {e}")))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .map_err(|_| LoginError::PasswordMismatch)
    })
    .await
    .map_err(|e| LoginError::PasswordParse(format!("Task panic: {e}")))?
}

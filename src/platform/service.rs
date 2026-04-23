use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::platform::{
    domain::PlatformRole,
    dto::UserCreatedResponse,
    types::{LoginError, UserCreateError},
};

#[derive(sqlx::FromRow)]
pub struct UserLoginInfo {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: PlatformRole,
}

pub async fn login(pool: &PgPool, email: &str, password: &str) -> Result<String, LoginError> {
    let email = email.trim().to_lowercase();

    let user = sqlx::query_as!(
        UserLoginInfo, // 'role as "role: _"' is needed because sqlx doesn't have knowledge about user defined types
        r#"SELECT id, email, password_hash, role as "role: _" FROM platform_user WHERE email = $1"#,
        email
    )
    .fetch_optional(pool)
    .await
    .map_err(LoginError::Database)?;

    let user = match user {
        Some(user) => user,
        None => {
            let _ = verify_password(
                password,
                "$argon2id$v=19$m=19456,t=2,p=1$UEViVXBNSThsbjJhSURLSg$o6V/wycFOBK3Th3a26vAwg",
            )
            .await;
            return Err(LoginError::UserNotFound(email));
        }
    };

    verify_password(password, &user.password_hash).await?;
    Ok("MockJWT".into())
}

pub async fn register_user(
    pool: &PgPool,
    email: &str,
    password: &str,
    role: PlatformRole,
) -> Result<UserCreatedResponse, UserCreateError> {
    let email = email.trim().to_lowercase();

    let hash = hash_password(password)
        .await
        .map_err(UserCreateError::PasswordHashing)?;

    let result = sqlx::query_as!(
        UserCreatedResponse,
        r#"
        INSERT INTO platform_user (email, password_hash, role)
        VALUES ($1, $2, $3)
        RETURNING id, email, role as "role: _", created_at
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
                Err(UserCreateError::EmailExists(email))
            } else if db_err.constraint() == Some("platform_user_single_owner_idx") {
                Err(UserCreateError::OwnerExists(email))
            } else {
                Err(UserCreateError::Database(sqlx::Error::Database(db_err)))
            }
        }
        Err(err) => Err(UserCreateError::Database(err)),
    }
}

pub async fn ensure_owner(pool: &PgPool) -> Result<(), anyhow::Error> {
    if owner_exists(pool).await? {
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

    match register_user(pool, &email, &password, PlatformRole::Owner).await {
        Ok(_) => {
            tracing::info!("Created platform owner: {email}");
            Ok(())
        }
        Err(UserCreateError::OwnerExists(_) | UserCreateError::EmailExists(_)) => {
            // This is more of a safeguard
            if owner_exists(pool).await? {
                tracing::warn!("Platform owner was created concurrently, continuing startup");
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to create owner"))
            }
        }
        Err(error) => Err(anyhow::anyhow!("Failed to create owner: {error}")),
    }
}

async fn owner_exists(pool: &PgPool) -> Result<bool, anyhow::Error> {
    let res =
        sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM platform_user WHERE role = 'owner')")
            .fetch_one(pool)
            .await
            .map_err(|err| anyhow::anyhow!("DB error: {err}"))?
            .unwrap_or(false);
    Ok(res)
}

async fn hash_password(password: &str) -> Result<String, anyhow::Error> {
    let password = password.to_string();

    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
    })
    .await
    .map_err(|e| anyhow::anyhow!("Password hashing task panicked: {e}"))?
    .map_err(|e| anyhow::anyhow!("Argon2 error: {e}"))
}

async fn verify_password(password: &str, stored_hash: &str) -> Result<(), LoginError> {
    let password = password.to_string();
    let stored_hash = stored_hash.to_string();

    tokio::task::spawn_blocking(move || {
        let hash = PasswordHash::new(&stored_hash).map_err(|e| LoginError::Other(e.to_string()))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .map_err(LoginError::PasswordMismatch)
    })
    .await
    .map_err(|e| LoginError::Other(e.to_string()))?
}

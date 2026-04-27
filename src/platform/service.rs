use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use super::{
    domain::PlatformRole,
    dto::{CreateUserRequest, CreateUserResponse},
    error::{BootstrapError, CreateUserError, LoginError},
};
use crate::{error::ProblemDetails, util::password};

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
            let dummy_result = password::verify(
                password,
                "$argon2id$v=19$m=19456,t=2,p=1$UEViVXBNSThsbjJhSURLSg$o6V/wycFOBK3Th3a26vAwg",
            )
            .await;
            let _ = std::hint::black_box(dummy_result);
            return Err(LoginError::UserNotFound(email.into()));
        }
    };

    match password::verify(password, &user.password_hash).await? {
        true => Ok(user),
        false => Err(LoginError::PasswordMismatch),
    }
}

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password: &str,
    role: PlatformRole,
) -> Result<CreateUserResponse, CreateUserError> {
    let hash = password::hash(password).await?;
    let result = sqlx::query_as!(
        CreateUserResponse,
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
                Err(CreateUserError::EmailExists(email.into()))
            } else if db_err.constraint() == Some("platform_user_single_owner_idx") {
                Err(CreateUserError::OwnerExists(email.into()))
            } else {
                Err(CreateUserError::Database(sqlx::Error::Database(db_err)))
            }
        }
        Err(err) => Err(CreateUserError::Database(err)),
    }
}

pub async fn ensure_owner(pool: &PgPool) -> Result<(), BootstrapError> {
    if owner_exists(pool).await? {
        tracing::info!("Platform owner exists, skipping bootstrap");
        return Ok(());
    }

    let req = CreateUserRequest {
        email: std::env::var("PLATFORM_OWNER_EMAIL")
            .map_err(|_| BootstrapError::MissingEmail)?
            .trim()
            .to_lowercase(),
        password: std::env::var("PLATFORM_OWNER_PASSWORD")
            .map_err(|_| BootstrapError::MissingPassword)?,
    };

    if let Err(err) = req.validate() {
        let problem = ProblemDetails::from(err);
        tracing::error!(
            error = %problem,
            "invalid platform owner configuration"
        );
        return Err(BootstrapError::Validation);
    }

    tracing::info!("No platform owner found, creating from environment");

    match create_user(pool, &req.email, &req.password, PlatformRole::Owner).await {
        Ok(_) => {
            tracing::info!("Created platform owner");
            Ok(())
        }
        Err(CreateUserError::OwnerExists(_) | CreateUserError::EmailExists(_)) => {
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

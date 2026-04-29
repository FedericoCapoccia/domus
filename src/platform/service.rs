use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use super::{
    domain::{PlatformRole, PlatformUserCredentials},
    dto::{CreateUserRequest, CreateUserResponse},
    error::{BootstrapError, CreateUserError, LoginError},
};
use crate::{
    error::ProblemDetails,
    platform::{domain::PlatformUser, error::GetUserError},
    util::password,
};

pub async fn login(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<PlatformUserCredentials, LoginError> {
    let user = sqlx::query_as!(
        PlatformUserCredentials, // 'role as "role: _"' is needed because sqlx doesn't have knowledge about user defined types
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
            return Err(LoginError::UserNotFound);
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
                Err(CreateUserError::EmailExists)
            } else if db_err.constraint() == Some("platform_user_single_owner_idx") {
                Err(CreateUserError::OwnerExists)
            } else {
                Err(CreateUserError::Database(sqlx::Error::Database(db_err)))
            }
        }
        Err(err) => Err(CreateUserError::Database(err)),
    }
}

pub async fn ensure_owner(
    pool: &PgPool,
    owner_email: Option<&str>,
    owner_password: Option<&str>,
) -> Result<(), BootstrapError> {
    if owner_exists(pool).await? {
        tracing::info!("Platform owner exists, skipping bootstrap");
        return Ok(());
    }

    let req = CreateUserRequest {
        email: owner_email
            .ok_or(BootstrapError::MissingEmail)?
            .trim()
            .to_lowercase(),
        password: owner_password
            .ok_or(BootstrapError::MissingPassword)?
            .to_string(),
        role: PlatformRole::Owner,
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

    match create_user(pool, &req.email, &req.password, req.role).await {
        Ok(_) => {
            tracing::info!("Created platform owner");
            Ok(())
        }
        Err(CreateUserError::OwnerExists | CreateUserError::EmailExists) => {
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

pub async fn get_user_by_id(pool: &PgPool, id: Uuid) -> Result<PlatformUser, GetUserError> {
    let user = sqlx::query_as!(
        PlatformUser,
        r#"
        SELECT
            id,
            email,
            role as "role: _",
            created_at,
            updated_at
        FROM platform_user
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;
    user.ok_or(GetUserError::NotFound)
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    const TEST_PASSWORD: &str = "password123";

    #[sqlx::test(migrations = "./migrations")]
    async fn ensure_owner_skips_bootstrap_when_owner_exists(pool: PgPool) {
        seed_owner(&pool, "owner@example.com").await;

        ensure_owner(&pool, None, None).await.unwrap();

        assert_eq!(owner_count(&pool).await, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn ensure_owner_returns_missing_email_when_no_owner_and_email_absent(pool: PgPool) {
        let result = ensure_owner(&pool, None, Some(TEST_PASSWORD)).await;

        assert!(matches!(result, Err(BootstrapError::MissingEmail)));
        assert_eq!(owner_count(&pool).await, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn ensure_owner_returns_missing_password_when_no_owner_and_password_absent(pool: PgPool) {
        let result = ensure_owner(&pool, Some("owner@example.com"), None).await;

        assert!(matches!(result, Err(BootstrapError::MissingPassword)));
        assert_eq!(owner_count(&pool).await, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn ensure_owner_returns_validation_error_for_invalid_owner_config(pool: PgPool) {
        let result = ensure_owner(&pool, Some("not-an-email"), Some("short")).await;

        assert!(matches!(result, Err(BootstrapError::Validation)));
        assert_eq!(owner_count(&pool).await, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn ensure_owner_creates_owner_from_bootstrap_config(pool: PgPool) {
        ensure_owner(&pool, Some(" OWNER@example.COM "), Some(TEST_PASSWORD))
            .await
            .unwrap();

        let owner = sqlx::query_as::<_, (String, String)>(
            "SELECT email, role::text FROM platform_user WHERE role = 'owner'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(owner.0, "owner@example.com");
        assert_eq!(owner.1, "owner");
        assert_eq!(owner_count(&pool).await, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_user_returns_owner_exists_for_second_owner(pool: PgPool) {
        create_user(
            &pool,
            "owner@example.com",
            TEST_PASSWORD,
            PlatformRole::Owner,
        )
        .await
        .unwrap();

        let result = create_user(
            &pool,
            "second-owner@example.com",
            TEST_PASSWORD,
            PlatformRole::Owner,
        )
        .await;

        assert!(matches!(result, Err(CreateUserError::OwnerExists)));
    }

    async fn seed_owner(pool: &PgPool, email: &str) {
        let hash = password::hash(TEST_PASSWORD).await.unwrap();

        sqlx::query(
            r#"
            INSERT INTO platform_user (email, password_hash, role)
            VALUES ($1, $2, 'owner')
            "#,
        )
        .bind(email)
        .bind(hash)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn owner_count(pool: &PgPool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM platform_user WHERE role = 'owner'")
            .fetch_one(pool)
            .await
            .unwrap()
    }
}

use sqlx::{Executor, Postgres};
use uuid::Uuid;

use super::{
    domain::{PlatformRole, PlatformStatus, PlatformUser, PlatformUserCredentials},
    dto::CreateUserResponse,
};

const ADVISORY_LOCK_NAMESPACE_DOMUS: i32 = 0x444f4d53; // "DOMS"
const ADVISORY_LOCK_PLATFORM_BOOTSTRAP: i32 = 1;

pub async fn platform_user_credentials_by_email<'e, E>(
    executor: E,
    email: &str,
) -> Result<Option<PlatformUserCredentials>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        PlatformUserCredentials,
        r#"
        SELECT
            id,
            password_hash,
            status as "status: _"
        FROM platform_user
        WHERE email = $1
        "#,
        email
    )
    .fetch_optional(executor)
    .await
}

pub async fn insert_platform_user<'e, E>(
    executor: E,
    email: &str,
    password_hash: &str,
    role: PlatformRole,
) -> Result<CreateUserResponse, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        CreateUserResponse,
        r#"
        INSERT INTO platform_user (email, password_hash, role)
        VALUES ($1, $2, $3)
        RETURNING id, role as "role: _", created_at
        "#,
        email,
        password_hash,
        role as PlatformRole,
    )
    .fetch_one(executor)
    .await
}

pub async fn owner_exists<'e, E>(executor: E) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    let res =
        sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM platform_user WHERE role = 'owner')")
            .fetch_one(executor)
            .await?
            .unwrap_or(false);
    Ok(res)
}

pub async fn platform_user_by_id<'e, E>(
    executor: E,
    id: Uuid,
) -> Result<Option<PlatformUser>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_as!(
        PlatformUser,
        r#"
        SELECT
            id,
            email,
            role as "role: _",
            status as "status: _",
            created_at,
            updated_at
        FROM platform_user
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(executor)
    .await
}

pub async fn update_platform_user_status<'e, E>(
    executor: E,
    id: Uuid,
    status: PlatformStatus,
) -> Result<Option<Uuid>, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_scalar!(
        r#"
        UPDATE platform_user
        SET
            status = $2,
            updated_at = now()
        WHERE id = $1
        RETURNING id
        "#,
        id,
        status as PlatformStatus,
    )
    .fetch_optional(executor)
    .await
}

pub async fn acquire_platform_bootstrap_lock<'e, E>(executor: E) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query("SELECT pg_advisory_xact_lock($1, $2)")
        .bind(ADVISORY_LOCK_NAMESPACE_DOMUS)
        .bind(ADVISORY_LOCK_PLATFORM_BOOTSTRAP)
        .execute(executor)
        .await?;
    Ok(())
}

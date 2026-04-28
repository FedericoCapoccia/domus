use axum::http::StatusCode;
use domus::api::platform::{CreateUserResponse, PlatformRole};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use super::helpers;

const TEST_PASSWORD: &str = "password123";
const TEST_EMAIL: &str = "user@example.com";

#[sqlx::test(migrations = "./migrations")]
async fn register_returns_201(pool: PgPool) {
    let token = helpers::platform_token(Uuid::now_v7(), PlatformRole::Admin);
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();
    let res = helpers::register_authed(&mut app, &token, &body).await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let body: CreateUserResponse = helpers::json_body(res).await;
    assert_eq!(body.role, PlatformRole::User);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_normalizes_email(pool: PgPool) {
    let token = helpers::platform_token(Uuid::now_v7(), PlatformRole::Admin);
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": "USER@example.COM",
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();

    let res = helpers::register_authed(&mut app, &token, &body).await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();

    let res = helpers::register_authed(&mut app, &token, &body).await;
    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_duplicate_email_returns_409(pool: PgPool) {
    let token = helpers::platform_token(Uuid::now_v7(), PlatformRole::Admin);
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();

    let res = helpers::register_authed(&mut app, &token, &body).await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = helpers::register_authed(&mut app, &token, &body).await;
    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_rejects_invalid_requests(pool: PgPool) {
    let token = helpers::platform_token(Uuid::now_v7(), PlatformRole::Admin);
    let mut app = helpers::app(pool);

    for body in [
        serde_json::json!({
            "email": "not-an-email",
            "password": TEST_PASSWORD,
            "role": "user",
        }),
        serde_json::json!({
            "email": TEST_EMAIL,
            "password": "short",
            "role": "user",
        }),
        serde_json::json!({
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
            "role": "random-role",
        }),
        serde_json::json!({
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
            "role": "owner",
            "foo": "bar",
        }),
    ] {
        let body = body.to_string();
        let response = helpers::register_authed(&mut app, &token, &body).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn owner_create_user_permissions(pool: PgPool) {
    assert_create_permissions(
        pool,
        PlatformRole::Owner,
        &[
            (PlatformRole::Owner, StatusCode::FORBIDDEN),
            (PlatformRole::Admin, StatusCode::CREATED),
            (PlatformRole::User, StatusCode::CREATED),
        ],
    )
    .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_create_user_permissions(pool: PgPool) {
    assert_create_permissions(
        pool,
        PlatformRole::Admin,
        &[
            (PlatformRole::Owner, StatusCode::FORBIDDEN),
            (PlatformRole::Admin, StatusCode::FORBIDDEN),
            (PlatformRole::User, StatusCode::CREATED),
        ],
    )
    .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn user_create_user_permissions(pool: PgPool) {
    assert_create_permissions(
        pool,
        PlatformRole::User,
        &[
            (PlatformRole::Owner, StatusCode::FORBIDDEN),
            (PlatformRole::Admin, StatusCode::FORBIDDEN),
            (PlatformRole::User, StatusCode::FORBIDDEN),
        ],
    )
    .await;
}

async fn assert_create_permissions(
    pool: PgPool,
    actor_role: PlatformRole,
    cases: &[(PlatformRole, StatusCode)],
) {
    let token = helpers::platform_token(Uuid::now_v7(), actor_role);
    let mut app = helpers::app(pool);
    for (target_role, expected_status) in cases {
        let body = json!({
            "email": &format!("{actor_role}-creates-{target_role}@example.com"),
            "password": TEST_PASSWORD,
            "role": target_role,
        })
        .to_string();
        let res = helpers::register_authed(&mut app, &token, &body).await;
        assert_eq!(
            res.status(),
            *expected_status,
            "actor_role={actor_role}, target_role={target_role}"
        );
        if *expected_status == StatusCode::CREATED {
            let body: CreateUserResponse = helpers::json_body(res).await;
            assert_eq!(body.role, *target_role);
        }
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn register_without_token_returns_401(pool: PgPool) {
    let mut app = helpers::app(pool);
    let body = json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();
    let res = helpers::register_without_token(&mut app, &body).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers()
            .get(axum::http::header::WWW_AUTHENTICATE)
            .unwrap()
            .to_str()
            .unwrap(),
        "Bearer"
    );
}
#[sqlx::test(migrations = "./migrations")]
async fn register_with_invalid_token_returns_401(pool: PgPool) {
    let mut app = helpers::app(pool);
    let body = json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();
    let res = helpers::register_authed(&mut app, "not-a-valid-jwt", &body).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers()
            .get(axum::http::header::WWW_AUTHENTICATE)
            .unwrap()
            .to_str()
            .unwrap(),
        "Bearer"
    );
}

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
    let res = helpers::register(&mut app, &token, &body).await;

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

    let res = helpers::register(&mut app, &token, &body).await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();

    let res = helpers::register(&mut app, &token, &body).await;
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

    let res = helpers::register(&mut app, &token, &body).await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = helpers::register(&mut app, &token, &body).await;
    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_forbidden_when_actor_lacks_permission(pool: PgPool) {
    let token = helpers::platform_token(Uuid::now_v7(), PlatformRole::User);
    let mut app = helpers::app(pool);
    let body = json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();

    let res = helpers::register(&mut app, &token, &body).await;

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

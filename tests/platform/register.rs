use axum::{
    body::Body,
    http::{Response, StatusCode},
};
use domus::api::platform::{CreateUserResponse, PlatformRole};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;

use super::helpers::{self, TEST_EMAIL, TEST_PASSWORD};

async fn endpoint(app: &mut axum::Router, token: &str, body: &str) -> Response<Body> {
    app.oneshot(helpers::json_request(
        "POST",
        "/api/v1/platform/users",
        Some(token),
        body,
    ))
    .await
    .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn success_returns_201(pool: PgPool) {
    let id =
        helpers::seed_platform_user(&pool, "admin@example.com", TEST_PASSWORD, "admin", "active")
            .await;
    let token = helpers::platform_token(id);
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();
    let res = endpoint(&mut app, &token, &body).await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let body: CreateUserResponse = helpers::json_body(res).await;
    assert_eq!(body.role, PlatformRole::User);
}

#[sqlx::test(migrations = "./migrations")]
async fn normalizes_email(pool: PgPool) {
    let id =
        helpers::seed_platform_user(&pool, "admin@example.com", TEST_PASSWORD, "admin", "active")
            .await;
    let token = helpers::platform_token(id);
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": "  USER@example.COM ",
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();

    let res = endpoint(&mut app, &token, &body).await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();

    let res = endpoint(&mut app, &token, &body).await;
    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn duplicate_email_returns_409(pool: PgPool) {
    let id =
        helpers::seed_platform_user(&pool, "admin@example.com", TEST_PASSWORD, "admin", "active")
            .await;
    let token = helpers::platform_token(id);
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();

    let res = endpoint(&mut app, &token, &body).await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = endpoint(&mut app, &token, &body).await;
    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn insufficient_permission_returns_403(pool: PgPool) {
    let id =
        helpers::seed_platform_user(&pool, "actor@example.com", TEST_PASSWORD, "user", "active")
            .await;
    let token = helpers::platform_token(id);
    let mut app = helpers::app(pool);
    let body = json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "role": PlatformRole::User,
    })
    .to_string();

    let res = endpoint(&mut app, &token, &body).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

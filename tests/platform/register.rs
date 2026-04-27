use axum::http::StatusCode;
use domus::api::platform::{CreateUserResponse, PlatformRole};
use sqlx::PgPool;

use super::helpers;

const TEST_PASSWORD: &str = "password123";
const TEST_EMAIL: &str = "user@example.com";

#[sqlx::test(migrations = "./migrations")]
async fn register_returns_201(pool: PgPool) {
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    })
    .to_string();
    let res = helpers::register(&mut app, &body).await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let body: CreateUserResponse = helpers::json_body(res).await;
    assert_eq!(body.role, PlatformRole::User);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_normalizes_email(pool: PgPool) {
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": "USER@example.COM",
        "password": TEST_PASSWORD,
    })
    .to_string();

    let res = helpers::register(&mut app, &body).await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    })
    .to_string();

    let res = helpers::register(&mut app, &body).await;
    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_duplicate_email_returns_409(pool: PgPool) {
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    })
    .to_string();

    let res = helpers::register(&mut app, &body).await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = helpers::register(&mut app, &body).await;
    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_rejects_invalid_requests(pool: PgPool) {
    let mut app = helpers::app(pool);

    for body in [
        serde_json::json!({
            "email": "not-an-email",
            "password": TEST_PASSWORD,
        }),
        serde_json::json!({
            "email": TEST_EMAIL,
            "password": "short",
        }),
        serde_json::json!({
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
            "role": "owner",
        }),
    ] {
        let body = body.to_string();
        let response = helpers::register(&mut app, &body).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

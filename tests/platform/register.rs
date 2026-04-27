use axum::{body::to_bytes, http::StatusCode};
use sqlx::PgPool;
use tower::ServiceExt;

use super::helpers::{self, json_request};

// TODO: refactor after auth middleware

#[sqlx::test(migrations = "./migrations")]
async fn register_returns_201(pool: PgPool) {
    let app = helpers::app(pool);

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/users",
            r#"{ "email": "new@example.com", "password": "password123" }"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["id"].as_str().is_some());
    assert_eq!(json["role"], "user");
}

#[sqlx::test(migrations = "./migrations")]
async fn register_normalizes_email(pool: PgPool) {
    let app = helpers::app(pool);

    app.clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/users",
            r#"{ "email": "  USER@Example.COM  ", "password": "password123" }"#,
        ))
        .await
        .unwrap();

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/users",
            r#"{ "email": "user@example.com", "password": "password123" }"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_duplicate_email_returns_409(pool: PgPool) {
    let app = helpers::app(pool);

    app.clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/users",
            r#"{ "email": "user@example.com", "password": "password123" }"#,
        ))
        .await
        .unwrap();

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/users",
            r#"{ "email": "user@example.com", "password": "differentpassword" }"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_rejects_invalid_email(pool: PgPool) {
    let app = helpers::app(pool);

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/users",
            r#"{ "email": "not-an-email", "password": "password123" }"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_rejects_short_password(pool: PgPool) {
    let app = helpers::app(pool);

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/users",
            r#"{ "email": "user@example.com", "password": "short" }"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_rejects_unknown_fields(pool: PgPool) {
    let app = helpers::app(pool);

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/users",
            r#"{ "email": "user@example.com", "password": "password123", "role": "owner" }"#,
        ))
        .await
        .unwrap();

    assert!(matches!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    ));
}

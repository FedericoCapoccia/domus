use axum::{body::to_bytes, http::StatusCode};
use domus::{PlatformRole, jwt};
use jsonwebtoken::DecodingKey;
use sqlx::PgPool;
use tower::ServiceExt;

use super::helpers::{self, json_request};

#[sqlx::test(migrations = "./migrations")]
async fn login_with_valid_credentials_returns_jwt(pool: PgPool) {
    let user_id =
        helpers::seed_platform_user(&pool, "user@example.com", "password123", "user").await;
    let app = helpers::app(pool);

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/login",
            r#"{ "email": "user@example.com", "password": "password123" }"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let token = json["token"].as_str().unwrap();
    let claims = jwt::verify(token, &DecodingKey::from_secret(helpers::JWT_SECRET)).unwrap();
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.iss, "domus");
    assert!(claims.iat <= claims.exp);
    assert_eq!(claims.nbf, claims.iat);
    assert!(claims.nbf <= claims.exp);
    assert!(claims.exp > time::OffsetDateTime::now_utc().unix_timestamp());
    assert!(matches!(
        claims.data,
        jwt::ClaimData::Platform {
            role: PlatformRole::User
        }
    ));
}

#[sqlx::test(migrations = "./migrations")]
async fn login_with_wrong_password_returns_401(pool: PgPool) {
    helpers::seed_platform_user(&pool, "user@example.com", "password123", "user").await;
    let app = helpers::app(pool);

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/login",
            r#"{ "email": "user@example.com", "password": "wrongpassword" }"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_with_unknown_email_returns_401(pool: PgPool) {
    let app = helpers::app(pool);

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/login",
            r#"{ "email": "unknown@example.com", "password": "password123" }"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_normalizes_email(pool: PgPool) {
    helpers::seed_platform_user(&pool, "user@example.com", "password123", "user").await;
    let app = helpers::app(pool);

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/login",
            r#"{ "email": "  user@EXAMPLE.COM  ", "password": "password123" }"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_rejects_invalid_email(pool: PgPool) {
    let app = helpers::app(pool);

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/login",
            r#"{ "email": "not-an-email", "password": "password123" }"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_rejects_short_password(pool: PgPool) {
    let app = helpers::app(pool);

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/login",
            r#"{ "email": "user@example.com", "password": "short" }"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_rejects_unknown_fields(pool: PgPool) {
    let app = helpers::app(pool);

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/v1/platform/login",
            r#"{ "email": "user@example.com", "password": "password123", "role": "owner" }"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

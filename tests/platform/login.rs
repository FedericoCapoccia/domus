use axum::http::StatusCode;
use domus::{api::platform::LoginResponse, jwt};
use jsonwebtoken::DecodingKey;
use sqlx::PgPool;

use super::helpers;

const TEST_PASSWORD: &str = "password123";
const TEST_EMAIL: &str = "user@example.com";

#[sqlx::test(migrations = "./migrations")]
async fn login_with_valid_credentials_returns_jwt(pool: PgPool) {
    let user_id = helpers::seed_platform_user(&pool, TEST_EMAIL, TEST_PASSWORD, "user").await;
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    })
    .to_string();
    let res = helpers::login(&mut app, &body).await;

    assert_eq!(res.status(), StatusCode::OK);

    let body: LoginResponse = helpers::json_body(res).await;
    let claims = jwt::verify(&body.token, &DecodingKey::from_secret(helpers::JWT_SECRET)).unwrap();

    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.iss, "domus");
    assert!(claims.iat <= claims.exp);
    assert_eq!(claims.nbf, claims.iat);
    assert!(claims.nbf <= claims.exp);
    assert!(claims.exp > time::OffsetDateTime::now_utc().unix_timestamp());
    assert!(matches!(claims.data, jwt::ClaimData::Platform));
}

#[sqlx::test(migrations = "./migrations")]
async fn login_with_wrong_password_returns_401(pool: PgPool) {
    helpers::seed_platform_user(&pool, TEST_EMAIL, TEST_PASSWORD, "user").await;
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": "wrong-password",
    })
    .to_string();
    let res = helpers::login(&mut app, &body).await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_with_unknown_email_returns_401(pool: PgPool) {
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    })
    .to_string();
    let res = helpers::login(&mut app, &body).await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_normalizes_email(pool: PgPool) {
    helpers::seed_platform_user(&pool, TEST_EMAIL, TEST_PASSWORD, "user").await;
    let mut app = helpers::app(pool);

    let body = serde_json::json!({
        "email": "  USER@example.COM ",
        "password": TEST_PASSWORD,
    })
    .to_string();
    let res = helpers::login(&mut app, &body).await;

    assert_eq!(res.status(), StatusCode::OK);
}

use axum::http::{StatusCode, header};
use domus::api::platform::{MeResponse, PlatformRole};
use sqlx::PgPool;
use uuid::Uuid;

use super::helpers;

const TEST_PASSWORD: &str = "password123";
const TEST_EMAIL: &str = "user@example.com";

#[sqlx::test(migrations = "./migrations")]
async fn me_returns_current_user(pool: PgPool) {
    let user_id = helpers::seed_platform_user(&pool, TEST_EMAIL, TEST_PASSWORD, "admin").await;
    let token = helpers::platform_token(user_id, PlatformRole::Admin);
    let mut app = helpers::app(pool);

    let res = helpers::me_authed(&mut app, &token).await;

    assert_eq!(res.status(), StatusCode::OK);

    let body: MeResponse = helpers::json_body(res).await;
    assert_eq!(body.id, user_id);
    assert_eq!(body.email, TEST_EMAIL);
    assert_eq!(body.role, PlatformRole::Admin);
}

#[sqlx::test(migrations = "./migrations")]
async fn me_without_token_returns_401(pool: PgPool) {
    let mut app = helpers::app(pool);

    let res = helpers::me_without_token(&mut app).await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers()
            .get(header::WWW_AUTHENTICATE)
            .unwrap()
            .to_str()
            .unwrap(),
        "Bearer"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn me_with_invalid_token_returns_401(pool: PgPool) {
    let mut app = helpers::app(pool);

    let res = helpers::me_authed(&mut app, "not-a-valid-jwt").await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers()
            .get(header::WWW_AUTHENTICATE)
            .unwrap()
            .to_str()
            .unwrap(),
        "Bearer"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn me_with_token_for_missing_user_returns_401(pool: PgPool) {
    let token = helpers::platform_token(Uuid::now_v7(), PlatformRole::Admin);
    let mut app = helpers::app(pool);

    let res = helpers::me_authed(&mut app, &token).await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers()
            .get(header::WWW_AUTHENTICATE)
            .unwrap()
            .to_str()
            .unwrap(),
        "Bearer"
    );
}

use axum::{
    body::Body,
    http::{Response, StatusCode},
};
use domus::api::platform::{MeResponse, PlatformRole, PlatformStatus};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use super::helpers::{self, TEST_EMAIL, TEST_PASSWORD};

async fn endpoint(app: &mut axum::Router, token: &str) -> Response<Body> {
    app.oneshot(helpers::json_request(
        "GET",
        "/api/v1/platform/me",
        Some(token),
        "",
    ))
    .await
    .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn returns_current_user(pool: PgPool) {
    let user_id =
        helpers::seed_platform_user(&pool, TEST_EMAIL, TEST_PASSWORD, "admin", "active").await;
    let token = helpers::platform_token(user_id);
    let mut app = helpers::app(pool);

    let res = endpoint(&mut app, &token).await;
    assert_eq!(res.status(), StatusCode::OK);

    let body: MeResponse = helpers::json_body(res).await;
    assert_eq!(body.id, user_id);
    assert_eq!(body.email, TEST_EMAIL);
    assert_eq!(body.role, PlatformRole::Admin);
    assert_eq!(body.status, PlatformStatus::Active);
}

#[sqlx::test(migrations = "./migrations")]
async fn missing_user_returns_401(pool: PgPool) {
    let token = helpers::platform_token(Uuid::now_v7());
    let mut app = helpers::app(pool);

    let res = endpoint(&mut app, &token).await;
    helpers::assert_bearer_unauthorized(res);
}

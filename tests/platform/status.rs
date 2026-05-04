use axum::{
    body::Body,
    http::{Response, StatusCode},
};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use super::helpers::{self, TEST_PASSWORD};

async fn endpoint(
    app: &mut axum::Router,
    token: &str,
    target_id: Uuid,
    action: &str,
) -> Response<Body> {
    app.oneshot(helpers::json_request(
        "POST",
        &format!("/api/v1/platform/users/{target_id}/{action}"),
        Some(token),
        "",
    ))
    .await
    .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn success_updates_status(pool: PgPool) {
    let admin_id =
        helpers::seed_platform_user(&pool, "admin@example.com", TEST_PASSWORD, "admin", "active")
            .await;
    let user_id =
        helpers::seed_platform_user(&pool, "user@example.com", TEST_PASSWORD, "user", "active")
            .await;
    let token = helpers::platform_token(admin_id);
    let mut app = helpers::app(pool.clone());

    let res = endpoint(&mut app, &token, user_id, "disable").await;

    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    assert_eq!(user_status(&pool, user_id).await, "disabled");
}

#[sqlx::test(migrations = "./migrations")]
async fn insufficient_permission_returns_403(pool: PgPool) {
    let actor_id =
        helpers::seed_platform_user(&pool, "actor@example.com", TEST_PASSWORD, "user", "active")
            .await;
    let target_id =
        helpers::seed_platform_user(&pool, "target@example.com", TEST_PASSWORD, "user", "active")
            .await;
    let token = helpers::platform_token(actor_id);
    let mut app = helpers::app(pool);

    let res = endpoint(&mut app, &token, target_id, "lock").await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn self_modification_returns_403(pool: PgPool) {
    let user_id =
        helpers::seed_platform_user(&pool, "user@example.com", TEST_PASSWORD, "admin", "active")
            .await;
    let token = helpers::platform_token(user_id);
    let mut app = helpers::app(pool);

    let res = endpoint(&mut app, &token, user_id, "lock").await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn missing_target_returns_404(pool: PgPool) {
    let admin_id =
        helpers::seed_platform_user(&pool, "admin@example.com", TEST_PASSWORD, "admin", "active")
            .await;
    let token = helpers::platform_token(admin_id);
    let mut app = helpers::app(pool);

    let res = endpoint(&mut app, &token, Uuid::now_v7(), "lock").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

async fn user_status(pool: &PgPool, id: Uuid) -> String {
    sqlx::query_scalar::<_, String>("SELECT status::text FROM platform_user WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

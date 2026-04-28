use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Response, header},
};
use domus::{AppState, api::platform::PlatformRole, build_router, jwt, password};
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

pub const JWT_SECRET: &[u8] = b"secret-that-is-at-least-32-bytes-long";

pub fn app(pool: PgPool) -> axum::Router {
    jwt::install_crypto_provider();

    build_router(AppState {
        pool,
        encoding_key: EncodingKey::from_secret(JWT_SECRET),
        decoding_key: DecodingKey::from_secret(JWT_SECRET),
    })
}

pub async fn seed_platform_user(pool: &PgPool, email: &str, password: &str, role: &str) -> Uuid {
    let hash = password::hash(password).await.unwrap();
    sqlx::query_scalar(
        r#"
        INSERT INTO platform_user (email, password_hash, role)
        VALUES ($1, $2, $3::platform_user_role)
        RETURNING id
        "#,
    )
    .bind(email)
    .bind(hash)
    .bind(role)
    .fetch_one(pool)
    .await
    .unwrap()
}

pub fn platform_token(user_id: Uuid, role: PlatformRole) -> String {
    jwt::generate(
        &jwt::Claims::platform(user_id, role),
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .unwrap()
}

pub async fn login(app: &mut axum::Router, body: &str) -> Response<Body> {
    app.oneshot(json_request("POST", "/api/v1/platform/login", None, body))
        .await
        .unwrap()
}

pub async fn register_authed(app: &mut axum::Router, token: &str, body: &str) -> Response<Body> {
    app.oneshot(json_request(
        "POST",
        "/api/v1/platform/users",
        Some(token),
        body,
    ))
    .await
    .unwrap()
}

pub async fn register_without_token(app: &mut axum::Router, body: &str) -> Response<Body> {
    app.oneshot(json_request("POST", "/api/v1/platform/users", None, body))
        .await
        .unwrap()
}

pub async fn me_authed(app: &mut axum::Router, token: &str) -> Response<Body> {
    app.oneshot(json_request("GET", "/api/v1/platform/me", Some(token), ""))
        .await
        .unwrap()
}

pub async fn me_without_token(app: &mut axum::Router) -> Response<Body> {
    app.oneshot(json_request("GET", "/api/v1/platform/me", None, ""))
        .await
        .unwrap()
}

fn json_request(method: &str, path: &str, token: Option<&str>, body: &str) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(path).header(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    if let Some(token) = token {
        builder = builder.header(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
    }

    builder.body(Body::from(body.to_string())).unwrap()
}

pub async fn json_body<T>(response: Response<Body>) -> T
where
    T: serde::de::DeserializeOwned,
{
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

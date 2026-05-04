use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Response, StatusCode, header},
};
use domus::{AppState, build_router, jwt, password};
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;
use uuid::Uuid;

pub const JWT_SECRET: &[u8] = b"secret-that-is-at-least-32-bytes-long";
pub const TEST_EMAIL: &str = "user@example.com";
pub const TEST_PASSWORD: &str = "password123";

pub fn app(pool: PgPool) -> axum::Router {
    jwt::install_crypto_provider();

    build_router(AppState {
        pool,
        encoding_key: EncodingKey::from_secret(JWT_SECRET),
        decoding_key: DecodingKey::from_secret(JWT_SECRET),
    })
}

pub async fn seed_platform_user(
    pool: &PgPool,
    email: &str,
    password: &str,
    role: &str,
    status: &str,
) -> Uuid {
    let hash = password::hash(password).await.unwrap();
    sqlx::query_scalar(
        r#"
        INSERT INTO platform_user (email, password_hash, role, status)
        VALUES ($1, $2, $3::platform_user_role, $4::platform_user_status)
        RETURNING id
        "#,
    )
    .bind(email)
    .bind(hash)
    .bind(role)
    .bind(status)
    .fetch_one(pool)
    .await
    .unwrap()
}

pub fn platform_token(user_id: Uuid) -> String {
    jwt::generate(
        &jwt::Claims::platform(user_id),
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .unwrap()
}

pub fn json_request(method: &str, path: &str, token: Option<&str>, body: &str) -> Request<Body> {
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

pub fn assert_unauthorized(response: Response<Body>) {
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

pub fn assert_bearer_unauthorized(response: Response<Body>) {
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response
            .headers()
            .get(header::WWW_AUTHENTICATE)
            .unwrap()
            .to_str()
            .unwrap(),
        "Bearer"
    );
}

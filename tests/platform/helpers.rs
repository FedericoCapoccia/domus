use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, header},
};
use domus::{AppState, build_router, util::password};
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

pub const JWT_SECRET: &[u8] = b"secret-that-is-at-least-32-bytes-long";

pub fn install_crypto_provider() {
    static INSTALL: std::sync::Once = std::sync::Once::new();
    INSTALL.call_once(|| {
        let _ = jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER.install_default();
    });
}

pub fn app(pool: PgPool) -> axum::Router {
    install_crypto_provider();

    build_router(AppState {
        pool,
        encoding_key: EncodingKey::from_secret(JWT_SECRET),
        _decoding_key: DecodingKey::from_secret(JWT_SECRET),
    })
}

pub fn json_request(method: &str, path: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )
        .body(Body::from(body.to_string()))
        .unwrap()
}

pub async fn seed_platform_user(pool: &PgPool, email: &str, password: &str, role: &str) {
    let hash = password::hash(password).await.unwrap();
    sqlx::query(
        r#"
        INSERT INTO platform_user (email, password_hash, role)
        VALUES ($1, $2, $3::platform_user_role)
        "#,
    )
    .bind(email)
    .bind(hash)
    .bind(role)
    .execute(pool)
    .await
    .unwrap();
}

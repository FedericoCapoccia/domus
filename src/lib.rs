mod app;
mod auth;
mod error;
mod extractors;
mod platform;
mod util;

use axum::{Router, extract::DefaultBodyLimit};
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::{PgPool, postgres::PgConnectOptions};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub async fn run() -> anyhow::Result<()> {
    jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER
        .install_default()
        .expect("Failed to install JWT crypto provider");

    let jwt_secret =
        std::env::var("JWT_SECRET").map_err(|_| anyhow::anyhow!("JWT_SECRET not set"))?;

    if jwt_secret.len() < 32 {
        return Err(anyhow::anyhow!("JWT_SECRET must be at least 32 bytes"));
    }

    let opts = PgConnectOptions::new()
        .host(&std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".into()))
        .port(5432) // TODO: add env
        .username(&std::env::var("POSTGRES_USER")?)
        .password(&std::env::var("POSTGRES_PASSWORD")?)
        .database(&std::env::var("POSTGRES_DB")?);

    let state = app::AppState {
        pool: PgPool::connect_with(opts).await?,
        encoding_key: EncodingKey::from_secret(jwt_secret.as_bytes()),
        decoding_key: DecodingKey::from_secret(jwt_secret.as_bytes()),
    };

    sqlx::migrate!().run(&state.pool).await?;

    platform::ensure_owner(&state.pool).await?;

    let router = Router::new()
        .nest("/api/v1/platform", platform::handler::router())
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;

    Ok(())
}

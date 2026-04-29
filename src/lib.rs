pub mod api {
    pub mod platform {
        pub use crate::platform::api::{
            CreateUserRequest, CreateUserResponse, LoginRequest, LoginResponse, MeResponse,
            PlatformRole,
        };
    }
}

pub mod jwt {
    pub use crate::auth::jwt::{
        ClaimData, Claims, JwtError, generate, install_crypto_provider, verify,
    };
}

pub mod password {
    pub use crate::util::password::{PasswordHashError, PasswordVerifyError, hash, verify};
}

pub use app::AppState;

mod app;
mod auth;
mod error;
mod extractors;
mod platform;
mod util;

use std::time::Duration;

use axum::{Router, extract::DefaultBodyLimit, http::StatusCode, routing::get};
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::platform::service::ensure_owner;

pub async fn run() -> anyhow::Result<()> {
    auth::jwt::install_crypto_provider();

    let jwt_secret =
        std::env::var("JWT_SECRET").map_err(|_| anyhow::anyhow!("JWT_SECRET not set"))?;

    if jwt_secret.len() < 32 {
        return Err(anyhow::anyhow!("JWT_SECRET must be at least 32 bytes"));
    }

    let opts = PgConnectOptions::new()
        .host(&std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".into()))
        .port(env_u16("POSTGRES_PORT", 5432)?)
        .username(&std::env::var("POSTGRES_USER")?)
        .password(&std::env::var("POSTGRES_PASSWORD")?)
        .database(&std::env::var("POSTGRES_DB")?);

    let state = app::AppState {
        pool: PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(env_u64(
                "POSTGRES_ACQUIRE_TIMEOUT_SECONDS",
                5,
            )?))
            .connect_with(opts)
            .await?,
        encoding_key: EncodingKey::from_secret(jwt_secret.as_bytes()),
        decoding_key: DecodingKey::from_secret(jwt_secret.as_bytes()),
    };

    sqlx::migrate!().run(&state.pool).await?;

    let owner_email = non_empty_env("PLATFORM_OWNER_EMAIL");
    let owner_password = non_empty_env("PLATFORM_OWNER_PASSWORD");
    ensure_owner(
        &state.pool,
        owner_email.as_deref(),
        owner_password.as_deref(),
    )
    .await?;

    let router = build_router(state);
    let bind_addr = format!("0.0.0.0:{}", env_u16("DOMUS_PORT", 3000)?);
    let listener = TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .nest("/api/v1/platform", platform::handler::router(state.clone()))
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn env_u16(name: &str, default: u16) -> anyhow::Result<u16> {
    match std::env::var(name) {
        Ok(value) => value
            .parse()
            .map_err(|_| anyhow::anyhow!("{name} must be a valid u16")),
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(err) => Err(err.into()),
    }
}

async fn healthz() -> StatusCode {
    StatusCode::NO_CONTENT
}

fn env_u64(name: &str, default: u64) -> anyhow::Result<u64> {
    match std::env::var(name) {
        Ok(value) => value
            .parse()
            .map_err(|_| anyhow::anyhow!("{name} must be a valid u64")),
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(err) => Err(err.into()),
    }
}

mod error;
mod platform;

use axum::Router;
use sqlx::{ConnectOptions, PgPool, postgres::PgConnectOptions};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

// TODO: add graceful shutdown and explicitly close pool https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#note-drop-behavior
// TODO: ensure that the plaform owner exists (platform_user with owner role). if not create it from env
//
// NOTE: rn the default user is federico@example.com with pwd: 'pallepalle'
//       $argon2i$v=19$m=16,t=2,p=1$dGt6MzFUYmc1U2hSWHRDbg$3Xn4v6reW1CPud/RaLYu1w
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    if let Err(e) = run().await {
        tracing::error!("Application error: {e:#}");
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let opts = PgConnectOptions::new()
        .host(&std::env::var("POSTGRES_HOST").unwrap_or_else(|_| String::from("localhost")))
        .port(5432)
        .username(&std::env::var("POSTGRES_USER")?)
        .password(&std::env::var("POSTGRES_PASSWORD")?)
        .database(&std::env::var("POSTGRES_DB")?)
        .log_statements(tracing::log::LevelFilter::Trace);

    let state = AppState {
        pool: PgPool::connect_with(opts).await?,
    };

    sqlx::migrate!().run(&state.pool).await?;
    platform::ensure_owner(state.pool.clone()).await?;

    let router = Router::new()
        .nest("/api/v1/platform", platform::handler::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;

    Ok(())
}

mod platform;

use axum::Router;
use sqlx::{ConnectOptions, PgPool, postgres::PgConnectOptions};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    let opts = PgConnectOptions::new()
        .host("localhost")
        .port(5432)
        .username(&std::env::var("POSTGRES_USER")?)
        .password(&std::env::var("POSTGRES_PASSWORD")?)
        .database(&std::env::var("POSTGRES_DB")?)
        .log_statements(tracing::log::LevelFilter::Trace);

    let pool = PgPool::connect_with(opts).await?;

    sqlx::migrate!().run(&pool).await?;

    let router = Router::new()
        .nest("/api/v1/platform", platform::handler::router())
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;

    Ok(())
}

use axum::{Router, routing::get};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(ok) => ok,
        Err(err) => {
            tracing::error!(error = %err, "Failed to read connection string from env");
            return;
        }
    };
    let pool = match PgPool::connect(&database_url).await {
        Ok(ok) => ok,
        Err(err) => {
            tracing::error!(error = %err, "Failed to connect to DB");
            return;
        }
    };

    match sqlx::migrate!().run(&pool).await {
        Ok(_) => tracing::info!("Migrations applied"),
        Err(err) => {
            tracing::error!(error = %err, "Failed to apply migrations");
            return;
        }
    }

    let router = Router::new().route("/", get(|| async { "Hello" }));

    let listener = match TcpListener::bind("0.0.0.0:3000").await {
        Ok(ok) => ok,
        Err(err) => {
            tracing::error!(error = %err, "Failed to bind TCP listener");
            return;
        }
    };

    if let Err(err) = axum::serve(listener, router).await {
        tracing::error!(error = %err, "Server failed");
        return;
    }
}

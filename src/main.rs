use axum::{Router, routing::get};
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

    tracing::info!("Hello World!");

    let router = Router::new().route("/", get("Hello"));

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

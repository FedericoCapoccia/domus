use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// TODO: add graceful shutdown and explicitly close pool https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#note-drop-behavior
// TODO: add rate limiting on /login
// TODO: add auth middleware
// TODO: TLS for postgres
// TODO: TLS or proxy
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    if let Err(e) = domus::run().await {
        tracing::error!("Application error: {e}");
        std::process::exit(1);
    }
}

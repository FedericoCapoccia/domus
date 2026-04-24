use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// TODO: add graceful shutdown and explicitly close pool https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#note-drop-behavior
// TODO: add rate limiting on /login
// TODO: add auth middleware
// TODO: TLS for postgres
// TODO: TLS or proxy
//
// TODO:
// Integration test plan:
// - Extract reusable `build_router(AppState)` from `src/lib.rs::run()`.
// - Add DB-backed tests in `tests/platform_auth.rs` using `#[sqlx::test(migrations = "./migrations")]`.
// - Test `POST /api/v1/platform/users`:
//   - valid registration returns `201`
//   - email is normalized before insert
//   - duplicate email returns `409`
//   - invalid email/password returns `422`
//   - unknown fields are rejected
// - Test `POST /api/v1/platform/login`:
//   - valid credentials return `200` with decodable JWT
//   - wrong password returns `401`
//   - unknown email returns `401`
//   - JWT contains expected `sub`, `iss`, `kind`, `role`, and `exp`
// - Test request/error handling:
//   - oversized body returns `413`
//   - malformed JSON returns `400`
//   - missing JSON content type returns `415`
// - Later: add bootstrap owner tests for `src/platform/service.rs::ensure_owner` after splitting env parsing from owner creation.

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

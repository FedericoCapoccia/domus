use axum::{Router, routing::post};

pub fn router() -> Router {
    Router::new().route("/login", post(login))
}

async fn login() -> &'static str {
    "Hello"
}

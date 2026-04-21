use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};

use crate::{
    AppState,
    error::ProblemDetails,
    platform::{service, types::LoginRequest},
};

pub fn router() -> Router<AppState> {
    Router::new().route("/login", post(login))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, ProblemDetails> {
    let jwt = service::login(state.pool, req).await?;
    Ok((StatusCode::OK, jwt))
}

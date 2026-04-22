use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};

use crate::{
    AppState,
    error::ProblemDetails,
    platform::{service, types::LoginRequest},
};

// NOTE:
// - DELETE /users/:id -> owner can delete all but himself, admin can delete role='user'
// - POST /users/:id/role -> owner can modify roles
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        // .route("/logout", post(async || {}))
        .route("/users", get(async || {}))
        .route("/users", post(async || {}))
    // .route("/users/id", get(async || {}))
    // .route("/users/id", delete(async || {}))
    // .route("/users/id/role", post(async || {}))
    // .route("/me", get(async || {}))
    // .route("/me", patch(async || {}))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, ProblemDetails> {
    let jwt = service::login(&state.pool, &req.email, &req.password).await?;
    Ok((StatusCode::OK, jwt))
}

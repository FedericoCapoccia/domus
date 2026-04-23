use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use validator::Validate;

use crate::{
    AppState,
    error::ProblemDetails,
    platform::{
        dto::{LoginRequest, UserCreateRequest},
        service,
    },
};

// NOTE:
// - DELETE /users/:id -> owner can delete all but himself, admin can delete role='user'
// - POST /users/:id/role -> owner can modify roles
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        // .route("/logout", post(async || {}))
        .route("/users", get(async || {}))
        .route("/users", post(register))
    // .route("/users/id", get(async || {}))
    // .route("/users/id", delete(async || {}))
    // .route("/users/id/role", post(async || {}))
    // .route("/me", get(async || {}))
    // .route("/me", patch(async || {}))
}

async fn login(
    State(state): State<AppState>,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ProblemDetails> {
    let Json(req) = payload.map_err(ProblemDetails::from)?;
    let jwt = service::login(&state.pool, &req.email, &req.password).await?;
    Ok((StatusCode::OK, jwt))
}

async fn register(
    State(state): State<AppState>,
    payload: Result<Json<UserCreateRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ProblemDetails> {
    let Json(req) = payload.map_err(ProblemDetails::from)?;
    req.validate().map_err(ProblemDetails::from)?;

    let created = service::register_user(
        &state.pool,
        &req.email,
        &req.password,
        super::domain::PlatformRole::User,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(created)))
}

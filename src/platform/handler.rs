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
    extractor::ValidatedJson,
    jwt,
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
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> Result<impl IntoResponse, ProblemDetails> {
    let user = service::login(&state.pool, &req.email, &req.password).await?;
    let token = jwt::generate(user.id, &user.email, user.role, &state.encoding_key, 15)
        .map_err(|_| ProblemDetails::internal_error())?;
    Ok((StatusCode::OK, Json(token)))
}

async fn register(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<UserCreateRequest>,
) -> Result<impl IntoResponse, ProblemDetails> {
    let created = service::register_user(
        &state.pool,
        &req.email,
        &req.password,
        super::domain::PlatformRole::User,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(created)))
}

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};

use super::{
    api::{CreateUserRequest, LoginRequest, LoginResponse, PlatformRole},
    service,
};
use crate::{
    app::AppState,
    auth::jwt::{self, Claims},
    error::ProblemDetails,
    extractors::validated_json::ValidatedJson,
};

// NOTE:
// - DELETE /users/:id -> owner can delete all but himself, admin can delete role='user'
// - POST /users/:id/role -> owner can modify roles
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        // .route("/logout", post(async || {}))
        // .route("/users", get(async || {}))
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

    let claims = Claims::platform(user.id, user.role);
    let res = LoginResponse {
        token: jwt::generate(&claims, &state.encoding_key)?,
    };

    Ok((StatusCode::OK, Json(res)))
}

async fn register(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<CreateUserRequest>,
) -> Result<impl IntoResponse, ProblemDetails> {
    let created =
        service::create_user(&state.pool, &req.email, &req.password, PlatformRole::User).await?;

    Ok((StatusCode::CREATED, Json(created)))
}

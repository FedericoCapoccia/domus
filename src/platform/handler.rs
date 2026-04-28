use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};

use super::{
    api::{CreateUserRequest, LoginRequest, LoginResponse, PlatformRole},
    service,
};
use crate::{
    app::AppState,
    auth::{
        jwt::{self, Claims},
        middleware::{PlatformAuth, require_platform_auth},
    },
    error::ProblemDetails,
    extractors::validated_json::ValidatedJson,
    platform::dto::MeResponse,
};

// NOTE:
// - DELETE /users/:id -> owner can delete all but himself, admin can delete role='user'
// - POST /users/:id/role -> owner can modify roles
// pub fn router() -> Router<AppState> {
// Router::new()
// .route("/login", post(login))
// .route("/logout", post(async || {}))
// .route("/users", get(async || {}))
// .route("/users/id", get(async || {}))
// .route("/users/id", delete(async || {}))
// .route("/users/id/role", post(async || {}))
// .route("/me", patch(async || {}))
// }

pub fn router(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/me", get(get_authenticated_user))
        .route("/users", post(register))
        .route_layer(middleware::from_fn_with_state(state, require_platform_auth));
    Router::new().route("/login", post(login)).merge(protected)
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
    Extension(auth): Extension<PlatformAuth>,
    ValidatedJson(req): ValidatedJson<CreateUserRequest>,
) -> Result<impl IntoResponse, ProblemDetails> {
    match (auth.role, req.role) {
        (PlatformRole::Owner, PlatformRole::Admin | PlatformRole::User) => {}
        (PlatformRole::Admin, PlatformRole::User) => {}
        (PlatformRole::Owner, PlatformRole::Owner) => {
            return Err(ProblemDetails::forbidden(
                "Cannot create a user with role=owner".into(),
            ));
        }
        _ => {
            return Err(ProblemDetails::forbidden(format!(
                "Insufficient permissions to create a user with role={0}",
                req.role
            )));
        }
    }

    let created = service::create_user(&state.pool, &req.email, &req.password, req.role).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

async fn get_authenticated_user(
    State(state): State<AppState>,
    Extension(auth): Extension<PlatformAuth>,
) -> Result<impl IntoResponse, ProblemDetails> {
    let user = service::get_user_by_id(&state.pool, auth.user_id).await?;
    Ok((StatusCode::OK, Json(MeResponse::from(user))))
}

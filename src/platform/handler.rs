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
        middleware::require_platform_auth,
    },
    error::ProblemDetails,
    extractors::validated_json::ValidatedJson,
    platform::{api::PlatformUser, dto::MeResponse},
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

    let claims = Claims::platform(user.id);
    let res = LoginResponse {
        token: jwt::generate(&claims, &state.encoding_key)?,
    };

    Ok((StatusCode::OK, Json(res)))
}

async fn register(
    State(state): State<AppState>,
    Extension(auth): Extension<PlatformUser>,
    ValidatedJson(req): ValidatedJson<CreateUserRequest>,
) -> Result<impl IntoResponse, ProblemDetails> {
    authorize_create_user(auth.role, req.role)?;

    let created = service::create_user(&state.pool, &req.email, &req.password, req.role).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

fn authorize_create_user(
    actor_role: PlatformRole,
    target_role: PlatformRole,
) -> Result<(), ProblemDetails> {
    match (actor_role, target_role) {
        (PlatformRole::Owner, PlatformRole::Admin | PlatformRole::User) => Ok(()),
        (PlatformRole::Admin, PlatformRole::User) => Ok(()),
        (PlatformRole::Owner, PlatformRole::Owner) => Err(ProblemDetails::forbidden(
            "Cannot create a user with role=owner".into(),
        )),
        _ => Err(ProblemDetails::forbidden(format!(
            "Insufficient permissions to create a user with role={target_role}"
        ))),
    }
}

async fn get_authenticated_user(
    Extension(auth): Extension<PlatformUser>,
) -> Result<impl IntoResponse, ProblemDetails> {
    Ok((StatusCode::OK, Json(MeResponse::from(auth))))
}

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::IntoResponse};

    use super::*;

    #[test]
    fn create_user_authorization_matches_role_matrix() {
        for (actor_role, target_role, expected_status) in [
            (
                PlatformRole::Owner,
                PlatformRole::Owner,
                StatusCode::FORBIDDEN,
            ),
            (PlatformRole::Owner, PlatformRole::Admin, StatusCode::OK),
            (PlatformRole::Owner, PlatformRole::User, StatusCode::OK),
            (
                PlatformRole::Admin,
                PlatformRole::Owner,
                StatusCode::FORBIDDEN,
            ),
            (
                PlatformRole::Admin,
                PlatformRole::Admin,
                StatusCode::FORBIDDEN,
            ),
            (PlatformRole::Admin, PlatformRole::User, StatusCode::OK),
            (
                PlatformRole::User,
                PlatformRole::Owner,
                StatusCode::FORBIDDEN,
            ),
            (
                PlatformRole::User,
                PlatformRole::Admin,
                StatusCode::FORBIDDEN,
            ),
            (
                PlatformRole::User,
                PlatformRole::User,
                StatusCode::FORBIDDEN,
            ),
        ] {
            let result = authorize_create_user(actor_role, target_role);
            let status = result
                .map(|_| StatusCode::OK)
                .unwrap_or_else(|problem| problem.into_response().status());

            assert_eq!(
                status, expected_status,
                "actor_role={actor_role}, target_role={target_role}"
            );
        }
    }
}

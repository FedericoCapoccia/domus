use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use uuid::Uuid;

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
    platform::{
        api::{PlatformStatus, PlatformUser},
        dto::MeResponse,
        error::GetUserError,
    },
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
        .route("/users/{id}/lock", post(lock_user))
        .route("/users/{id}/enable", post(enable_user))
        .route("/users/{id}/disable", post(disable_user))
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

fn authorize_create_user(actor: PlatformRole, target: PlatformRole) -> Result<(), ProblemDetails> {
    match (actor, target) {
        (PlatformRole::Owner, PlatformRole::Admin | PlatformRole::User) => Ok(()),
        (PlatformRole::Admin, PlatformRole::User) => Ok(()),
        (PlatformRole::Owner, PlatformRole::Owner) => Err(ProblemDetails::forbidden(
            "Cannot create a user with role=owner".into(),
        )),
        _ => Err(ProblemDetails::forbidden(format!(
            "Insufficient permissions to create a user with role={target}"
        ))),
    }
}

async fn get_authenticated_user(
    Extension(auth): Extension<PlatformUser>,
) -> Result<impl IntoResponse, ProblemDetails> {
    Ok((StatusCode::OK, Json(MeResponse::from(auth))))
}

async fn lock_user(
    State(state): State<AppState>,
    Extension(actor): Extension<PlatformUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ProblemDetails> {
    set_user_status_handler(state, actor, id, PlatformStatus::Locked).await
}
async fn disable_user(
    State(state): State<AppState>,
    Extension(actor): Extension<PlatformUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ProblemDetails> {
    set_user_status_handler(state, actor, id, PlatformStatus::Disabled).await
}
async fn enable_user(
    State(state): State<AppState>,
    Extension(actor): Extension<PlatformUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ProblemDetails> {
    set_user_status_handler(state, actor, id, PlatformStatus::Active).await
}

async fn set_user_status_handler(
    state: AppState,
    actor: PlatformUser,
    target_id: Uuid,
    status: PlatformStatus,
) -> Result<impl IntoResponse, ProblemDetails> {
    let target = match service::get_user_by_id(&state.pool, target_id).await {
        Ok(target) => target,
        Err(GetUserError::NotFound) => {
            return Err(ProblemDetails::not_found("User not found".into()));
        }
        Err(GetUserError::Database(internal)) => {
            tracing::error!(
                internal = ?internal,
                "target user lookup failed"
            );
            return Err(ProblemDetails::internal_error());
        }
    };
    authorize_set_user_status(actor.id, actor.role, target.id, target.role)?;
    service::set_user_status(&state.pool, target.id, status).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn authorize_set_user_status(
    actor_id: Uuid,
    actor_role: PlatformRole,
    target_id: Uuid,
    target_role: PlatformRole,
) -> Result<(), ProblemDetails> {
    if actor_id == target_id {
        return Err(ProblemDetails::forbidden(
            "Cannot modify your own status".into(),
        ));
    }
    match (actor_role, target_role) {
        (PlatformRole::Owner, PlatformRole::Admin | PlatformRole::User) => Ok(()),
        (PlatformRole::Admin, PlatformRole::User) => Ok(()),
        _ => Err(ProblemDetails::forbidden(
            "Insufficient permissions to modify user status".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::IntoResponse};
    use uuid::Uuid;

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

    #[test]
    fn set_user_status_authorization_matches_role_matrix() {
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
            let result =
                authorize_set_user_status(Uuid::now_v7(), actor_role, Uuid::now_v7(), target_role);
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

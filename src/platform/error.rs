use crate::{
    error::ProblemDetails,
    util::password::{PasswordHashError, PasswordVerifyError},
};

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("User not found")]
    UserNotFound(String /* email */),
    #[error("Password does not match")]
    PasswordMismatch,
    #[error("Password verification failed")]
    VerifyError(#[from] PasswordVerifyError),
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl From<LoginError> for ProblemDetails {
    fn from(err: LoginError) -> Self {
        match &err {
            LoginError::Database(internal) => {
                tracing::error!(
                    error = %err,
                    internal = ?internal,
                    "login failed"
                );
                ProblemDetails::internal_error()
            }
            LoginError::VerifyError(internal) => {
                tracing::error!(
                    error = %err,
                    internal = %internal,
                    "login failed"
                );
                ProblemDetails::internal_error()
            }
            LoginError::UserNotFound(_) | LoginError::PasswordMismatch => {
                tracing::warn!("login failed due to invalid credentials");
                ProblemDetails::unauthorized("Invalid credentials".into())
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UserCreateError {
    #[error("Email already exists")]
    EmailExists(String /* email */),
    #[error("Owner already exists")]
    OwnerExists(String /* email */),
    #[error("Failed to hash password")]
    HashingError(#[from] PasswordHashError),
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl From<UserCreateError> for ProblemDetails {
    fn from(err: UserCreateError) -> Self {
        match &err {
            UserCreateError::EmailExists(_) => {
                ProblemDetails::conflict("Email already exists".into())
            }
            UserCreateError::OwnerExists(_) => {
                tracing::error!(
                    error = %err,
                    "user creation failed"
                );
                ProblemDetails::internal_error()
            }
            UserCreateError::HashingError(internal) => {
                tracing::error!(
                    error = %err,
                    internal = %internal,
                    "user creation failed"
                );
                ProblemDetails::internal_error()
            }
            UserCreateError::Database(internal) => {
                tracing::error!(
                    error = %err,
                    internal = ?internal,
                    "user creation failed"
                );
                ProblemDetails::internal_error()
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error("PLATFORM_OWNER_EMAIL not set")]
    MissingEmail,
    #[error("PLATFORM_OWNER_PASSWORD not set")]
    MissingPassword,
    #[error("Invalid platform owner configuration")]
    Validation,
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Failed to create owner")]
    CreateFailed,
}

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::IntoResponse};

    use crate::error::ProblemDetails;

    use super::{LoginError, UserCreateError};

    #[test]
    fn user_not_found_maps_to_unauthorized_problem() {
        let response = ProblemDetails::from(LoginError::UserNotFound("user@example.com".into()))
            .into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn password_mismatch_maps_to_unauthorized_problem() {
        let response = ProblemDetails::from(LoginError::PasswordMismatch).into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn email_exists_maps_to_conflict_problem() {
        let response =
            ProblemDetails::from(UserCreateError::EmailExists("user@example.com".into()))
                .into_response();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}

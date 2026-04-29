use crate::{
    error::ProblemDetails,
    util::password::{PasswordHashError, PasswordVerifyError},
};

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("User not found")]
    UserNotFound,
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
            LoginError::UserNotFound | LoginError::PasswordMismatch => {
                ProblemDetails::unauthorized("Invalid credentials".into())
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateUserError {
    #[error("Email already exists")]
    EmailExists,
    #[error("Owner already exists")]
    OwnerExists,
    #[error("Failed to hash password")]
    HashingError(#[from] PasswordHashError),
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl From<CreateUserError> for ProblemDetails {
    fn from(err: CreateUserError) -> Self {
        match &err {
            CreateUserError::EmailExists => ProblemDetails::conflict("Email already exists".into()),
            CreateUserError::OwnerExists => {
                tracing::error!(
                    error = %err,
                    "user creation failed"
                );
                ProblemDetails::internal_error()
            }
            CreateUserError::HashingError(internal) => {
                tracing::error!(
                    error = %err,
                    internal = %internal,
                    "user creation failed"
                );
                ProblemDetails::internal_error()
            }
            CreateUserError::Database(internal) => {
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
    #[error("PLATFORM_OWNER_EMAIL already belongs to an existing user")]
    EmailExists,
    #[error("Failed to create owner")]
    CreateFailed,
}

#[derive(Debug, thiserror::Error)]
pub enum GetUserError {
    #[error("User not found")]
    NotFound,
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl From<GetUserError> for ProblemDetails {
    fn from(err: GetUserError) -> Self {
        match &err {
            GetUserError::NotFound => {
                ProblemDetails::bearer_unauthorized("Invalid or missing access token".into())
            }
            GetUserError::Database(internal) => {
                tracing::error!(
                    error = %err,
                    internal = ?internal,
                    "get user failed"
                );
                ProblemDetails::internal_error()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::IntoResponse};

    use super::*;
    use crate::util::password::{PasswordHashError, PasswordVerifyError};

    #[test]
    fn user_not_found_maps_to_unauthorized_problem() {
        assert_problem_status(
            ProblemDetails::from(LoginError::UserNotFound),
            StatusCode::UNAUTHORIZED,
        );
    }

    #[test]
    fn password_mismatch_maps_to_unauthorized_problem() {
        assert_problem_status(
            ProblemDetails::from(LoginError::PasswordMismatch),
            StatusCode::UNAUTHORIZED,
        );
    }

    #[test]
    fn login_internal_errors_map_to_internal_server_error_problem() {
        for problem in [
            ProblemDetails::from(LoginError::Database(sqlx::Error::RowNotFound)),
            ProblemDetails::from(LoginError::VerifyError(PasswordVerifyError::Parse(
                "invalid hash".into(),
            ))),
        ] {
            assert_problem_status(problem, StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    #[test]
    fn email_exists_maps_to_conflict_problem() {
        assert_problem_status(
            ProblemDetails::from(CreateUserError::EmailExists),
            StatusCode::CONFLICT,
        );
    }

    #[test]
    fn create_user_internal_errors_map_to_internal_server_error_problem() {
        for problem in [
            ProblemDetails::from(CreateUserError::OwnerExists),
            ProblemDetails::from(CreateUserError::HashingError(PasswordHashError::Hash(
                "hash failed".into(),
            ))),
            ProblemDetails::from(CreateUserError::Database(sqlx::Error::RowNotFound)),
        ] {
            assert_problem_status(problem, StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    #[test]
    fn get_user_errors_map_to_expected_problem_statuses() {
        assert_problem_status(
            ProblemDetails::from(GetUserError::NotFound),
            StatusCode::UNAUTHORIZED,
        );
        assert_problem_status(
            ProblemDetails::from(GetUserError::Database(sqlx::Error::RowNotFound)),
            StatusCode::INTERNAL_SERVER_ERROR,
        );
    }

    fn assert_problem_status(problem: ProblemDetails, status: StatusCode) {
        let response = problem.into_response();
        assert_eq!(response.status(), status);
    }
}

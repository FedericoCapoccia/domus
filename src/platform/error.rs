use crate::error::ProblemDetails;

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("User not found")]
    UserNotFound(String /* email */),
    #[error("Password does not match")]
    PasswordMismatch,
    #[error("Couldn't parse the password hash")]
    PasswordParse(String),
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
            LoginError::PasswordParse(internal) => {
                tracing::error!(
                    error = %err,
                    internal = %internal,
                    "login failed"
                );
                ProblemDetails::internal_error()
            }
            LoginError::UserNotFound(_) | LoginError::PasswordMismatch => {
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
    PasswordHashing(String),
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

impl From<UserCreateError> for ProblemDetails {
    fn from(err: UserCreateError) -> Self {
        match &err {
            UserCreateError::EmailExists(_) => {
                ProblemDetails::conflict("Email already exists".into())
            }
            UserCreateError::OwnerExists(_) => ProblemDetails::internal_error(),
            UserCreateError::PasswordHashing(internal) => {
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
    #[error("Password must be 8-128 characters")]
    InvalidPasswordLength,
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Failed to create owner")]
    CreateFailed,
}

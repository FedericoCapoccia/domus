use std::fmt::Display;

use crate::error::ProblemDetails;

#[derive(Debug)]
pub enum LoginError {
    UserNotFound,
    PasswordMismatch,
    Database(sqlx::Error),
    Other(String),
}

impl Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginError::UserNotFound => write!(f, "User not found"),
            LoginError::PasswordMismatch => {
                write!(f, "Password does not match")
            }
            LoginError::Database(error) => write!(f, "Query error: {error}"),
            LoginError::Other(message) => write!(f, "{message}"),
        }
    }
}

impl From<LoginError> for ProblemDetails {
    fn from(err: LoginError) -> Self {
        match err {
            LoginError::UserNotFound | LoginError::PasswordMismatch => {
                ProblemDetails::unauthorized("Invalid credentials".into())
            }
            LoginError::Other(_) | LoginError::Database(_) => ProblemDetails::internal_error(),
        }
    }
}

#[derive(Debug)]
pub enum UserCreateError {
    EmailExists(String),
    OwnerExists(String),
    PasswordHashing(anyhow::Error),
    Database(sqlx::Error),
}

impl Display for UserCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserCreateError::EmailExists(email) => write!(f, "Email '{email}' already exists"),
            UserCreateError::OwnerExists(email) => write!(f, "Owner '{email}' already exists"),
            UserCreateError::PasswordHashing(err) => write!(f, "Failed to hash password: {err}"),
            UserCreateError::Database(error) => write!(f, "Query error: {error}"),
        }
    }
}

impl From<UserCreateError> for ProblemDetails {
    fn from(err: UserCreateError) -> Self {
        match err {
            UserCreateError::EmailExists(_) => {
                ProblemDetails::conflict("Email already exists".into())
            }
            // I want to return 500 on OwnerExists to safeguard owner's email if this gets leaked
            // into an handler somehow
            UserCreateError::PasswordHashing(_)
            | UserCreateError::Database(_)
            | UserCreateError::OwnerExists(_) => ProblemDetails::internal_error(),
        }
    }
}

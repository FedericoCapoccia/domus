use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ProblemDetails;

//=====================================================================================================================
// Request DTO
//=====================================================================================================================

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

//=====================================================================================================================
// Response DTO
//=====================================================================================================================

//=====================================================================================================================
// Model DTO
//=====================================================================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type)]
#[sqlx(type_name = "platform_user_role", rename_all = "lowercase")]
pub enum PlatformRole {
    Owner,
    Admin,
    User,
}

#[derive(sqlx::FromRow)]
pub struct UserLoginInfo {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: PlatformRole,
}

#[derive(Serialize)]
pub struct UserCreatedResponse {
    pub id: Uuid,
    pub email: String,
    pub role: PlatformRole,
    pub created_at: DateTime<Utc>,
}

//=====================================================================================================================
// Errors
//=====================================================================================================================

#[derive(Debug)]
pub enum LoginError {
    UserNotFound(String),
    PasswordParsing,
    PasswordMismatch(argon2::password_hash::Error),
    Database(sqlx::Error),
}

impl Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginError::UserNotFound(email) => write!(f, "User '{email}' not found"),
            LoginError::PasswordParsing => {
                write!(f, "Failed to parse password from db")
            }
            LoginError::PasswordMismatch(error) => {
                write!(f, "Password does not match. err: {error}")
            }
            LoginError::Database(error) => write!(f, "Query error: {error}"),
        }
    }
}

impl From<LoginError> for ProblemDetails {
    fn from(err: LoginError) -> Self {
        match err {
            LoginError::UserNotFound(_) | LoginError::PasswordMismatch(_) => {
                ProblemDetails::unauthorized(String::from("Invalid credentials"))
            }
            LoginError::PasswordParsing | LoginError::Database(_) => {
                ProblemDetails::internal_error()
            }
        }
    }
}

#[derive(Debug)]
pub enum UserCreateError {
    EmailExists(String),
    OwnerExists(String),
    PasswordHashing(argon2::password_hash::Error),
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
                ProblemDetails::conflict(String::from("Email already exists"))
            }
            // I want to return 500 on OwnerExists to safeguard owner's email if this gets leaked
            // into an handler somehow
            UserCreateError::PasswordHashing(_)
            | UserCreateError::Database(_)
            | UserCreateError::OwnerExists(_) => ProblemDetails::internal_error(),
        }
    }
}

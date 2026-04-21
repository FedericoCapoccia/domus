use std::fmt::Display;

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
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: PlatformRole,
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
                ProblemDetails::internal_error(String::from("An unexpected error occurred"))
            }
        }
    }
}

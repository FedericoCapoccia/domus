use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use super::domain::{PlatformRole, PlatformStatus, PlatformUser};
use crate::util::serde::deserialize_normalized_email;

#[derive(Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    #[serde(deserialize_with = "deserialize_normalized_email")]
    #[validate(email, length(max = 254))]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    #[serde(deserialize_with = "deserialize_normalized_email")]
    #[validate(email, length(max = 254))]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    pub role: PlatformRole,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub id: Uuid,
    pub role: PlatformRole,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub email: String,
    pub role: PlatformRole,
    pub status: PlatformStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<PlatformUser> for MeResponse {
    fn from(value: PlatformUser) -> Self {
        Self {
            id: value.id,
            email: value.email,
            role: value.role,
            status: value.status,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_request_rejects_invalid_email() {
        let req = LoginRequest {
            email: "not-an-email".into(),
            ..valid_login_request()
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn login_request_rejects_short_password() {
        let req = LoginRequest {
            password: "short".into(),
            ..valid_login_request()
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn login_request_rejects_unknown_fields() {
        let result = serde_json::from_value::<LoginRequest>(serde_json::json!({
            "email": "user@example.com",
            "password": "password123",
            "role": "owner",
        }));
        assert!(result.is_err());
    }

    #[test]
    fn create_user_request_rejects_invalid_email() {
        let req = CreateUserRequest {
            email: "not-an-email".into(),
            ..valid_create_user_request()
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_user_request_rejects_short_password() {
        let req = CreateUserRequest {
            password: "short".into(),
            ..valid_create_user_request()
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_user_request_rejects_unknown_fields() {
        let result = serde_json::from_value::<CreateUserRequest>(serde_json::json!({
            "email": "user@example.com",
            "password": "password123",
            "role": "user",
            "foo": "bar",
        }));

        assert!(result.is_err());
    }

    #[test]
    fn create_user_request_rejects_unknown_role() {
        let result = serde_json::from_value::<CreateUserRequest>(serde_json::json!({
            "email": "user@example.com",
            "password": "password123",
            "role": "random-role",
        }));

        assert!(result.is_err());
    }

    fn valid_login_request() -> LoginRequest {
        LoginRequest {
            email: "user@example.com".into(),
            password: "password123".into(),
        }
    }

    fn valid_create_user_request() -> CreateUserRequest {
        CreateUserRequest {
            email: "user@example.com".into(),
            password: "password123".into(),
            role: PlatformRole::User,
        }
    }
}

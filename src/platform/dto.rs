use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::{platform::domain::PlatformRole, util::serde::deserialize_normalized_email};

#[derive(Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    #[serde(deserialize_with = "deserialize_normalized_email")]
    #[validate(email, length(max = 254))]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserCreateRequest {
    #[serde(deserialize_with = "deserialize_normalized_email")]
    #[validate(email, length(max = 254))]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Serialize)]
pub struct UserCreatedResponse {
    pub id: Uuid,
    pub role: PlatformRole,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use super::UserCreateRequest;
    use validator::Validate;

    #[test]
    fn registration_normalizes_email() {
        let req: UserCreateRequest = serde_json::from_str(
            r#"{ "email": "  USER@Example.COM  ", "password": "password123" }"#,
        )
        .unwrap();

        assert_eq!(req.email, "user@example.com");
    }

    #[test]
    fn registration_rejects_invalid_email() {
        let req: UserCreateRequest =
            serde_json::from_str(r#"{ "email": "not-an-email", "password": "password123" }"#)
                .unwrap();
        let err = req.validate().unwrap_err();
        assert!(err.field_errors().contains_key("email"));
    }

    #[test]
    fn registration_rejects_short_password() {
        let req: UserCreateRequest =
            serde_json::from_str(r#"{ "email": "user@example.com", "password": "short" }"#)
                .unwrap();
        let err = req.validate().unwrap_err();
        assert!(err.field_errors().contains_key("password"));
    }

    #[test]
    fn registration_rejects_unknown_fields() {
        let result = serde_json::from_str::<UserCreateRequest>(
            r#"{ "email": "user@example.com", "password": "password123", "role": "owner" }"#,
        );
        assert!(result.is_err());
    }
}

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use super::domain::PlatformRole;
use crate::{platform::domain::PlatformUser, util::serde::deserialize_normalized_email};

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
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

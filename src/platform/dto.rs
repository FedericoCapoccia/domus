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

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::platform::domain::PlatformRole;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserCreateRequest {
    #[validate(email, length(max = 254))]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Serialize)]
pub struct UserCreatedResponse {
    pub id: Uuid,
    pub email: String,
    pub role: PlatformRole,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::platform::domain::PlatformRole;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserCreatedResponse {
    pub id: Uuid,
    pub email: String,
    pub role: PlatformRole,
    pub created_at: OffsetDateTime,
}

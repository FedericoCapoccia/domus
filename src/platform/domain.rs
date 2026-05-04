use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type, Deserialize)]
#[sqlx(type_name = "platform_user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PlatformRole {
    Owner,
    Admin,
    User,
}

impl fmt::Display for PlatformRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Owner => f.write_str("owner"),
            Self::Admin => f.write_str("admin"),
            Self::User => f.write_str("user"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type, Deserialize)]
#[sqlx(type_name = "platform_user_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PlatformStatus {
    Active,
    Disabled,
    Locked,
}

#[derive(sqlx::FromRow)]
pub struct PlatformUserCredentials {
    pub id: Uuid,
    pub password_hash: String,
    pub status: PlatformStatus,
}

#[derive(sqlx::FromRow, Clone)]
pub struct PlatformUser {
    pub id: Uuid,
    pub email: String,
    pub role: PlatformRole,
    pub status: PlatformStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

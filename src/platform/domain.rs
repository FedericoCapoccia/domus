use serde::{Deserialize, Serialize};
use std::fmt;

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

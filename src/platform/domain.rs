use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, sqlx::Type, Deserialize)]
#[sqlx(type_name = "platform_user_role", rename_all = "lowercase")]
pub enum PlatformRole {
    Owner,
    Admin,
    User,
}

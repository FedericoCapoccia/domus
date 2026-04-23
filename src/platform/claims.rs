use serde::{Deserialize, Serialize};

use crate::platform::PlatformRole;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
    pub nbf: i64,

    pub email: String,
    pub role: PlatformRole,
}

impl Claims {
    pub fn new(sub: String, exp: i64, email: &str, role: PlatformRole) -> Self {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        Self {
            sub,
            iat: now,
            exp: now + exp * 60,
            iss: "Domus".into(),
            nbf: now,
            email: email.into(),
            role,
        }
    }
}

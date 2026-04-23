use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

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
        let now = OffsetDateTime::now_utc().unix_timestamp();
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

pub fn generate(
    user_id: Uuid,
    email: &str,
    role: PlatformRole,
    encoding_key: &EncodingKey,
    exp: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(user_id.to_string(), exp, email, role);
    encode(&Header::default(), &claims, encoding_key)
}

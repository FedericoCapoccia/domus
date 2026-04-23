use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::PlatformRole;

// TODO: return JwtError
// pub enum JwtError {
//     Generation(jsonwebtoken::errors::Error),
//     Verification(jsonwebtoken::errors::Error),
//     Expired,
//     Invalid,
// }
// - Generation(_) → 500
// - Verification(_) | Expired | Invalid → 401

#[derive(Serialize)]
pub struct JwtResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ClaimData {
    Platform { role: PlatformRole },
    Tenant { tenant_slug: String },
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub iss: String,
    pub iat: i64,
    pub nbf: i64,
    pub exp: i64,
    #[serde(flatten)]
    pub data: ClaimData,
}

impl Claims {
    pub fn platform(sub: Uuid, role: PlatformRole, minutes: i64) -> Self {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        Self {
            sub,
            iss: "domus".into(),
            iat: now,
            nbf: now,
            exp: now + minutes * 60,
            data: ClaimData::Platform { role },
        }
    }
    pub fn _tenant(sub: Uuid, tenant_slug: String, minutes: i64) -> Self {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        Self {
            sub,
            iss: "domus".into(),
            iat: now,
            nbf: now,
            exp: now + minutes * 60,
            data: ClaimData::Tenant { tenant_slug },
        }
    }
}

pub fn generate(
    claims: &Claims,
    encoding_key: &EncodingKey,
) -> Result<JwtResponse, jsonwebtoken::errors::Error> {
    Ok(JwtResponse {
        token: encode(&Header::default(), &claims, encoding_key)?,
    })
}

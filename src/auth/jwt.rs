use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ProblemDetails, platform::PlatformRole};

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

pub fn generate(claims: &Claims, encoding_key: &EncodingKey) -> Result<JwtResponse, JwtError> {
    Ok(JwtResponse {
        token: encode(&Header::default(), claims, encoding_key).map_err(JwtError::Generation)?,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Failed to generate JWT")]
    Generation(jsonwebtoken::errors::Error),
    #[error("JWT expired")]
    _Expired(jsonwebtoken::errors::Error),
    #[error("JWT invalid")]
    _Invalid(jsonwebtoken::errors::Error),
}

impl From<JwtError> for ProblemDetails {
    fn from(err: JwtError) -> Self {
        match &err {
            JwtError::Generation(internal) => {
                tracing::error!(
                    error = %err,
                    internal = ?internal,
                    "jwt generation"
                );
                ProblemDetails::internal_error()
            }
            JwtError::_Expired(internal) => {
                tracing::warn!(
                    error = %err,
                    internal = ?internal,
                    "jwt verification"
                );
                ProblemDetails::bearer_unauthorized("Invalid or missing access token".into())
            }
            JwtError::_Invalid(internal) => {
                tracing::warn!(
                    error = %err,
                    internal = ?internal,
                    "jwt verification"
                );
                ProblemDetails::bearer_unauthorized("Invalid or missing access token".into())
            }
        }
    }
}

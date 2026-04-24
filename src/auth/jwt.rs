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

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation, decode};
    use uuid::Uuid;

    use crate::platform::PlatformRole;

    use super::{ClaimData, Claims, generate};

    static INSTALL_PROVIDER: Once = Once::new();

    #[test]
    fn platform_claims_include_expected_registered_and_custom_fields() {
        let sub = Uuid::now_v7();
        let before = time::OffsetDateTime::now_utc().unix_timestamp();

        let claims = Claims::platform(sub, PlatformRole::Admin, 15);

        let after = time::OffsetDateTime::now_utc().unix_timestamp();
        assert_eq!(claims.sub, sub);
        assert_eq!(claims.iss, "domus");
        assert!(claims.iat >= before);
        assert!(claims.iat <= after);
        assert_eq!(claims.nbf, claims.iat);
        assert_eq!(claims.exp, claims.iat + 15 * 60);
        assert!(matches!(
            claims.data,
            ClaimData::Platform {
                role: PlatformRole::Admin
            }
        ));
    }

    #[test]
    fn platform_claims_serialize_with_platform_kind() {
        let claims = Claims::platform(Uuid::now_v7(), PlatformRole::Owner, 15);

        let serialized = serde_json::to_value(&claims).unwrap();

        assert_eq!(serialized["kind"], "platform");
        assert_eq!(serialized["role"], "owner");
    }

    #[test]
    fn generate_returns_decodable_token() {
        install_crypto_provider();
        let secret = b"secret-that-is-at-least-32-bytes-long";
        let sub = Uuid::now_v7();
        let claims = Claims::platform(sub, PlatformRole::User, 15);

        let response = generate(&claims, &EncodingKey::from_secret(secret)).unwrap();

        let token = decode::<Claims>(
            &response.token,
            &DecodingKey::from_secret(secret),
            &Validation::new(Algorithm::HS256),
        )
        .unwrap();

        assert_eq!(token.claims.sub, sub);
        assert_eq!(token.claims.iss, "domus");
        assert!(matches!(
            token.claims.data,
            ClaimData::Platform {
                role: PlatformRole::User
            }
        ));
    }

    fn install_crypto_provider() {
        INSTALL_PROVIDER.call_once(|| {
            jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER
                .install_default()
                .expect("failed to install JWT crypto provider");
        });
    }
}

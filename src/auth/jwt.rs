use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ProblemDetails, platform::api::PlatformRole};

const ISSUER: &str = "domus";
const PLATFORM_ACCESS_TOKEN_TTL: time::Duration = time::Duration::minutes(15);
static INSTALL_PROVIDER: std::sync::Once = std::sync::Once::new();

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ClaimData {
    Platform { role: PlatformRole },
    Tenant { tenant_slug: String },
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub fn platform(sub: Uuid, role: PlatformRole) -> Self {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();

        Self {
            sub,
            iss: ISSUER.into(),
            iat: now,
            nbf: now,
            exp: now + PLATFORM_ACCESS_TOKEN_TTL.whole_seconds(),
            data: ClaimData::Platform { role },
        }
    }
    fn tenant(sub: Uuid, tenant_slug: String) -> Self {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();

        Self {
            sub,
            iss: ISSUER.into(),
            iat: now,
            nbf: now,
            exp: now + PLATFORM_ACCESS_TOKEN_TTL.whole_seconds(),
            data: ClaimData::Tenant { tenant_slug },
        }
    }
}

pub fn generate(claims: &Claims, encoding_key: &EncodingKey) -> Result<String, JwtError> {
    encode(&Header::default(), claims, encoding_key).map_err(JwtError::Generation)
}

pub fn verify(token: &str, decoding_key: &DecodingKey) -> Result<Claims, JwtError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[ISSUER]);

    decode::<Claims>(token, decoding_key, &validation)
        .map(|token| token.claims)
        .map_err(JwtError::Invalid)
}

pub fn install_crypto_provider() {
    INSTALL_PROVIDER.call_once(|| {
        jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER
            .install_default()
            .expect("failed to install JWT crypto provider");
    });
}

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Failed to generate JWT")]
    Generation(jsonwebtoken::errors::Error),
    #[error("JWT invalid")]
    Invalid(jsonwebtoken::errors::Error),
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
            JwtError::Invalid(internal) => {
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
    use super::*;

    #[test]
    fn platform_claims_include_expected_registered_and_custom_fields() {
        let sub = Uuid::now_v7();
        let before = time::OffsetDateTime::now_utc().unix_timestamp();

        let claims = Claims::platform(sub, PlatformRole::Admin);

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
        let claims = Claims::platform(Uuid::now_v7(), PlatformRole::Owner);

        let serialized = serde_json::to_value(&claims).unwrap();

        assert_eq!(serialized["kind"], "platform");
        assert_eq!(serialized["role"], "owner");
    }

    #[test]
    fn generate_returns_decodable_token() {
        install_crypto_provider();
        let secret = b"secret-that-is-at-least-32-bytes-long";
        let sub = Uuid::now_v7();
        let claims = Claims::platform(sub, PlatformRole::User);

        let response = generate(&claims, &EncodingKey::from_secret(secret)).unwrap();

        let claims = verify(&response, &DecodingKey::from_secret(secret)).unwrap();
        assert_eq!(claims.sub, sub);
        assert_eq!(claims.iss, "domus");
        assert!(matches!(
            claims.data,
            ClaimData::Platform {
                role: PlatformRole::User
            }
        ));
    }

    #[test]
    fn verify_rejects_token_signed_with_different_secret() {
        install_crypto_provider();
        let claims = Claims::platform(Uuid::now_v7(), PlatformRole::User);
        let token = generate(
            &claims,
            &EncodingKey::from_secret(b"secret-that-is-at-least-32-bytes-long"),
        )
        .unwrap();

        let err = verify(
            &token,
            &DecodingKey::from_secret(b"different-secret-that-is-long-enough"),
        )
        .unwrap_err();

        assert!(matches!(err, JwtError::Invalid(_)));
    }

    #[test]
    fn verify_rejects_expired_token() {
        install_crypto_provider();
        let secret = b"secret-that-is-at-least-32-bytes-long";
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let claims = Claims {
            sub: Uuid::now_v7(),
            iss: ISSUER.into(),
            iat: now - 3600,
            nbf: now - 3600,
            exp: now - 1800,
            data: ClaimData::Platform {
                role: PlatformRole::User,
            },
        };
        let token = generate(&claims, &EncodingKey::from_secret(secret)).unwrap();

        let err = super::verify(&token, &DecodingKey::from_secret(secret)).unwrap_err();

        assert!(matches!(err, JwtError::Invalid(_)));
    }

    #[test]
    fn verify_rejects_token_with_wrong_issuer() {
        install_crypto_provider();
        let secret = b"secret-that-is-at-least-32-bytes-long";
        let mut claims = Claims::platform(Uuid::now_v7(), PlatformRole::User);
        claims.iss = "not-domus".into();
        let token = generate(&claims, &EncodingKey::from_secret(secret)).unwrap();

        let err = super::verify(&token, &DecodingKey::from_secret(secret)).unwrap_err();

        assert!(matches!(err, JwtError::Invalid(_)));
    }

    #[test]
    fn verify_rejects_malformed_token() {
        install_crypto_provider();

        let err = super::verify(
            "not-a-jwt",
            &DecodingKey::from_secret(b"secret-that-is-at-least-32-bytes-long"),
        )
        .unwrap_err();

        assert!(matches!(err, JwtError::Invalid(_)));
    }

    #[test]
    fn tenant_claims_serialize_with_tenant_kind() {
        let claims = Claims::tenant(Uuid::now_v7(), "acme".into());

        let serialized = serde_json::to_value(&claims).unwrap();

        assert_eq!(serialized["kind"], "tenant");
        assert_eq!(serialized["tenant_slug"], "acme");
    }
}

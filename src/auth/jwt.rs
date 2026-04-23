use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;

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

pub fn generate<T: Serialize>(
    claims: &T,
    encoding_key: &EncodingKey,
) -> Result<JwtResponse, jsonwebtoken::errors::Error> {
    Ok(JwtResponse {
        token: encode(&Header::default(), &claims, encoding_key)?,
    })
}

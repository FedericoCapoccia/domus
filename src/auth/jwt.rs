use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;

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

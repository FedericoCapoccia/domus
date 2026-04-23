#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub encoding_key: jsonwebtoken::EncodingKey,
    pub _decoding_key: jsonwebtoken::DecodingKey,
}

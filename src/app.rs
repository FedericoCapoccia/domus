#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub encoding_key: jsonwebtoken::EncodingKey,
    pub decoding_key: jsonwebtoken::DecodingKey,
}

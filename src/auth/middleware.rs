use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use super::jwt::{self, ClaimData};
use crate::{AppState, api::platform::PlatformRole, error::ProblemDetails};

#[derive(Clone, Debug)]
pub struct PlatformAuth {
    pub user_id: uuid::Uuid,
    pub role: PlatformRole,
}

pub async fn require_platform_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ProblemDetails> {
    let token = bearer_token(&req)?;
    let claims = jwt::verify(token, &state.decoding_key)?;
    let ClaimData::Platform { role } = claims.data else {
        return Err(ProblemDetails::bearer_unauthorized(
            "Invalid or missing access token".into(),
        ));
    };
    req.extensions_mut().insert(PlatformAuth {
        user_id: claims.sub,
        role,
    });
    Ok(next.run(req).await)
}

fn bearer_token(req: &Request) -> Result<&str, ProblemDetails> {
    let header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            ProblemDetails::bearer_unauthorized("Invalid or missing access token".into())
        })?;
    header.strip_prefix("Bearer ").ok_or_else(|| {
        ProblemDetails::bearer_unauthorized("Invalid or missing access token".into())
    })
}

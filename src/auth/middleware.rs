use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use super::jwt::{self, ClaimData};
use crate::{AppState, error::ProblemDetails, platform::service};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TenantAuth {
    pub user_id: uuid::Uuid,
    pub tenant_slug: String,
}

pub async fn require_platform_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ProblemDetails> {
    let token = bearer_token(&req)?;
    let claims = jwt::verify(token, &state.decoding_key)?;
    let ClaimData::Platform = claims.data else {
        return Err(ProblemDetails::bearer_unauthorized(
            "Invalid or missing access token".into(),
        ));
    };
    let user = service::get_user_by_id(&state.pool, claims.sub).await?;
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

#[allow(dead_code)]
pub async fn require_tenant_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ProblemDetails> {
    let token = bearer_token(&req)?;
    let claims = jwt::verify(token, &state.decoding_key)?;
    let ClaimData::Tenant { tenant_slug } = claims.data else {
        return Err(ProblemDetails::bearer_unauthorized(
            "Invalid or missing access token".into(),
        ));
    };
    req.extensions_mut().insert(TenantAuth {
        user_id: claims.sub,
        tenant_slug,
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

#[cfg(test)]
mod tests {
    use axum::{
        Extension, Router,
        body::Body,
        http::{Request, StatusCode, header},
        middleware,
        response::Response,
        routing::get,
    };
    use jsonwebtoken::{DecodingKey, EncodingKey};
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;
    use crate::auth::jwt;

    const JWT_SECRET: &[u8] = b"secret-that-is-at-least-32-bytes-long";

    #[tokio::test]
    async fn protected_platform_route_without_token_returns_401() {
        let response = protected_platform_app()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_unauthorized(response);
    }

    #[tokio::test]
    async fn protected_platform_route_with_invalid_token_returns_401() {
        let response = protected_platform_app()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(header::AUTHORIZATION, "Bearer not-a-valid-jwt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_unauthorized(response);
    }

    #[tokio::test]
    async fn protected_platform_route_with_non_bearer_authorization_returns_401() {
        let response = protected_platform_app()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(header::AUTHORIZATION, "Basic abc123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_unauthorized(response);
    }

    #[tokio::test]
    async fn protected_platform_route_with_tenant_token_returns_401() {
        let token = tenant_token(Uuid::now_v7(), "acme");

        let response = protected_platform_app()
            .oneshot(authorized_request(&token))
            .await
            .unwrap();

        assert_unauthorized(response);
    }

    #[tokio::test]
    async fn protected_tenant_route_with_tenant_token_returns_204() {
        let user_id = Uuid::now_v7();
        let token = tenant_token(user_id, "acme");

        let response = protected_tenant_app()
            .oneshot(authorized_request(&token))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn protected_tenant_route_with_platform_token_returns_401() {
        let token = platform_token(Uuid::now_v7());

        let response = protected_tenant_app()
            .oneshot(authorized_request(&token))
            .await
            .unwrap();

        assert_unauthorized(response);
    }

    #[tokio::test]
    async fn protected_tenant_route_without_token_returns_401() {
        let response = protected_tenant_app()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_unauthorized(response);
    }

    #[tokio::test]
    async fn protected_tenant_route_with_invalid_token_returns_401() {
        let response = protected_tenant_app()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(header::AUTHORIZATION, "Bearer not-a-valid-jwt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_unauthorized(response);
    }

    #[tokio::test]
    async fn protected_tenant_route_with_non_bearer_authorization_returns_401() {
        let response = protected_tenant_app()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(header::AUTHORIZATION, "Basic abc123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_unauthorized(response);
    }

    async fn platform_handler() -> StatusCode {
        StatusCode::NO_CONTENT
    }

    async fn tenant_handler(Extension(auth): Extension<TenantAuth>) -> StatusCode {
        assert_eq!(auth.tenant_slug, "acme");
        StatusCode::NO_CONTENT
    }

    fn protected_platform_app() -> Router {
        jwt::install_crypto_provider();
        let state = app_state();

        Router::new()
            .route("/protected", get(platform_handler))
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                require_platform_auth,
            ))
            .with_state(state)
    }

    fn protected_tenant_app() -> Router {
        jwt::install_crypto_provider();
        let state = app_state();

        Router::new()
            .route("/protected", get(tenant_handler))
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                require_tenant_auth,
            ))
            .with_state(state)
    }

    fn app_state() -> AppState {
        AppState {
            pool: PgPoolOptions::new().connect_lazy_with(PgConnectOptions::new()),
            encoding_key: EncodingKey::from_secret(JWT_SECRET),
            decoding_key: DecodingKey::from_secret(JWT_SECRET),
        }
    }

    fn authorized_request(token: &str) -> Request<Body> {
        Request::builder()
            .uri("/protected")
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap()
    }

    fn platform_token(user_id: Uuid) -> String {
        jwt::generate(
            &jwt::Claims::platform(user_id),
            &EncodingKey::from_secret(JWT_SECRET),
        )
        .unwrap()
    }

    fn tenant_token(user_id: Uuid, tenant_slug: &str) -> String {
        jwt::generate(
            &jwt::Claims::tenant(user_id, tenant_slug.into()),
            &EncodingKey::from_secret(JWT_SECRET),
        )
        .unwrap()
    }

    fn assert_unauthorized(response: Response) {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response
                .headers()
                .get(header::WWW_AUTHENTICATE)
                .unwrap()
                .to_str()
                .unwrap(),
            "Bearer"
        );
    }
}

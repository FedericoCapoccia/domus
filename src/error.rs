use std::fmt::Display;

use axum::{
    body::Body,
    extract::rejection::JsonRejection,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FieldError {
    field: String,
    code: String,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    type_: String,
    title: String,
    status: u16,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<FieldError>>,
    #[serde(skip)]
    www_authenticate: Option<&'static str>,
}

impl ProblemDetails {
    pub fn new(
        status: StatusCode,
        title: String,
        detail: String,
        errors: Option<Vec<FieldError>>,
    ) -> Self {
        Self {
            type_: "about:blank".into(),
            title,
            status: status.as_u16(),
            detail,
            errors,
            www_authenticate: None,
        }
    }

    pub fn unauthorized(detail: String) -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "Unauthorized".into(),
            detail,
            None,
        )
    }

    pub fn bearer_unauthorized(detail: String) -> Self {
        let mut problem = Self::unauthorized(detail);
        problem.www_authenticate = Some("Bearer");
        problem
    }

    pub fn forbidden(detail: String) -> Self {
        Self::new(StatusCode::FORBIDDEN, "Forbidden".into(), detail, None)
    }

    pub fn not_found(detail: String) -> Self {
        Self::new(StatusCode::NOT_FOUND, "Not Found".into(), detail, None)
    }

    pub fn internal_error() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".into(),
            "An unexpected error occurred".into(),
            None,
        )
    }

    pub fn unprocessable_entity(detail: String, errors: Option<Vec<FieldError>>) -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Unprocessable Entity".into(),
            detail,
            errors,
        )
    }

    pub fn conflict(detail: String) -> Self {
        Self::new(StatusCode::CONFLICT, "Conflict".into(), detail, None)
    }

    pub fn bad_request(detail: String) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "Bad Request".into(), detail, None)
    }

    pub fn unsupported_media_type(detail: String) -> Self {
        Self::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported Media Type".into(),
            detail,
            None,
        )
    }
}

impl Display for ProblemDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self)
            .expect("ProblemDetails contains only infallible serializable fields");
        f.write_str(&json)
    }
}

impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let mut response = Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/problem+json");

        if let Some(value) = self.www_authenticate {
            response = response.header(header::WWW_AUTHENTICATE, value);
        };

        response
            .body(Body::from(
                serde_json::to_string(&self).expect("ProblemDetails is serializable"),
            ))
            .unwrap()
    }
}

impl From<validator::ValidationErrors> for ProblemDetails {
    fn from(err: validator::ValidationErrors) -> Self {
        let errors = err
            .field_errors()
            .into_iter()
            .flat_map(|(field, rules)| {
                rules.iter().map(move |rule| FieldError {
                    field: field.to_string(),
                    code: rule.code.to_string(),
                    message: rule.message.clone().map_or_else(
                        || format!("Validation failed for field '{}'", field),
                        |m| m.to_string(),
                    ),
                })
            })
            .collect();
        ProblemDetails::unprocessable_entity("Validation failed".into(), Some(errors))
    }
}

impl From<JsonRejection> for ProblemDetails {
    fn from(err: JsonRejection) -> Self {
        let status = err.status();

        match &err {
            JsonRejection::JsonDataError(_) => ProblemDetails::unprocessable_entity(
                "Request body has missing or invalid fields".into(),
                None,
            ),
            JsonRejection::JsonSyntaxError(_) => {
                ProblemDetails::bad_request("Malformed JSON body".into())
            }
            JsonRejection::MissingJsonContentType(_) => ProblemDetails::unsupported_media_type(
                "Expected 'Content-Type: application/json'".into(),
            ),
            JsonRejection::BytesRejection(_) => {
                let detail = if status == StatusCode::PAYLOAD_TOO_LARGE {
                    "Request body exceeds the maximum allowed size"
                } else {
                    "Failed to read request body"
                };

                ProblemDetails::new(
                    status,
                    status.canonical_reason().unwrap_or("Request Error").into(),
                    detail.into(),
                    None,
                )
            }
            _ => ProblemDetails::bad_request("Failed to parse JSON body".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        Json, Router,
        body::Body,
        extract::{DefaultBodyLimit, rejection::JsonRejection},
        routing::post,
    };
    use serde::Deserialize;
    use tower::ServiceExt;
    use validator::{Validate, ValidationErrors};

    use super::*;

    #[derive(Validate)]
    struct TestPayload {
        #[validate(email)]
        email: String,
    }

    #[derive(Deserialize, Validate)]
    struct TestPayloadWithCustomMessage {
        #[validate(email(message = "Email must be valid"))]
        email: String,
    }

    #[derive(Deserialize)]
    struct TestJsonPayload {
        email: String,
    }

    #[test]
    fn unauthorized_response_has_problem_json_status_and_content_type() {
        let response = ProblemDetails::unauthorized("Invalid credentials".into()).into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap(),
            "application/problem+json"
        );
    }

    #[test]
    fn bearer_unauthorized_response_sets_www_authenticate_header() {
        let response = ProblemDetails::bearer_unauthorized("Invalid token".into()).into_response();

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

    #[test]
    fn conflict_response_has_conflict_status() {
        let response = ProblemDetails::conflict("Email already exists".into()).into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn internal_error_response_has_internal_server_error_status() {
        let response = ProblemDetails::internal_error().into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn bad_request_response_has_bad_request_status() {
        let response = ProblemDetails::bad_request("Malformed JSON body".into()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn unsupported_media_type_response_has_unsupported_media_type_status() {
        let response = ProblemDetails::unsupported_media_type(
            "Expected 'Content-Type: application/json'".into(),
        )
        .into_response();
        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[test]
    fn validation_errors_convert_to_unprocessable_entity_problem() {
        let errors = validation_errors();

        let problem = ProblemDetails::from(errors);

        assert_eq!(problem.status, StatusCode::UNPROCESSABLE_ENTITY.as_u16());
        assert_eq!(problem.title, "Unprocessable Entity");
        assert_eq!(problem.detail, "Validation failed");
        assert_eq!(problem.errors.as_ref().unwrap().len(), 1);
        assert_eq!(problem.errors.as_ref().unwrap()[0].field, "email");
        assert_eq!(problem.errors.as_ref().unwrap()[0].code, "email");
    }

    #[test]
    fn display_serializes_problem_details_as_json() {
        let problem = ProblemDetails::from(validation_errors());
        let serialized: serde_json::Value = serde_json::from_str(&problem.to_string()).unwrap();

        assert_eq!(serialized["title"], "Unprocessable Entity");
        assert_eq!(
            serialized["status"],
            StatusCode::UNPROCESSABLE_ENTITY.as_u16()
        );
        assert_eq!(serialized["detail"], "Validation failed");

        let errors = serialized["errors"].as_array().unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0]["field"], "email");
        assert_eq!(errors[0]["code"], "email");
        assert!(errors[0]["message"].is_string());
    }

    #[test]
    fn validation_errors_use_custom_messages_when_present() {
        let errors = TestPayloadWithCustomMessage {
            email: "not-an-email".into(),
        }
        .validate()
        .unwrap_err();

        let problem = ProblemDetails::from(errors);

        assert_eq!(
            problem.errors.as_ref().unwrap()[0].message,
            "Email must be valid"
        );
    }

    #[tokio::test]
    async fn json_data_error_maps_to_unprocessable_entity() {
        let response = json_app()
            .oneshot(json_request(r#"{"email":123}"#))
            .await
            .unwrap();

        let status = response.status();
        let body = response_body(response).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_problem_title(&body, "Unprocessable Entity");
    }

    #[tokio::test]
    async fn json_syntax_error_maps_to_bad_request() {
        let response = json_app().oneshot(json_request("{")).await.unwrap();

        let status = response.status();
        let body = response_body(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_problem_title(&body, "Bad Request");
    }

    #[tokio::test]
    async fn missing_json_content_type_maps_to_unsupported_media_type() {
        let response = json_app()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/")
                    .body(Body::from(r#"{"email":"user@example.com"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = response_body(response).await;

        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_problem_title(&body, "Unsupported Media Type");
    }

    #[tokio::test]
    async fn payload_too_large_maps_to_payload_too_large() {
        let response = json_app()
            .layer(DefaultBodyLimit::max(8))
            .oneshot(json_request(r#"{"email":"user@example.com"}"#))
            .await
            .unwrap();

        let status = response.status();
        let body = response_body(response).await;

        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_problem_title(&body, "Payload Too Large");
    }

    fn validation_errors() -> ValidationErrors {
        TestPayload {
            email: "not-an-email".into(),
        }
        .validate()
        .unwrap_err()
    }

    fn json_app() -> Router {
        Router::new().route("/", post(json_handler))
    }

    async fn json_handler(
        payload: Result<Json<TestJsonPayload>, JsonRejection>,
    ) -> Result<StatusCode, ProblemDetails> {
        let Json(payload) = payload.map_err(ProblemDetails::from)?;
        let _ = payload.email;
        Ok(StatusCode::NO_CONTENT)
    }

    fn json_request(body: &'static str) -> axum::http::Request<Body> {
        axum::http::Request::builder()
            .method("POST")
            .uri("/")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    async fn response_body(response: Response) -> serde_json::Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    fn assert_problem_title(body: &serde_json::Value, title: &str) {
        assert_eq!(body["title"], title);
    }
}

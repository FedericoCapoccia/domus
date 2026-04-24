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
        match serde_json::to_string(self) {
            Ok(json) => f.write_str(&json),
            Err(_) => f.write_str("failed to serialize problem details"),
        }
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
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use serde::Deserialize;
    use validator::{Validate, ValidationErrors};

    use super::ProblemDetails;

    #[derive(Debug, Deserialize)]
    struct SerializedProblemDetails {
        title: String,
        status: u16,
        detail: String,
        errors: Option<Vec<SerializedFieldError>>,
    }

    #[derive(Debug, Deserialize)]
    struct SerializedFieldError {
        field: String,
        code: String,
        message: String,
    }

    #[derive(Validate)]
    struct TestPayload {
        #[validate(email)]
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
        let serialized: SerializedProblemDetails =
            serde_json::from_str(&problem.to_string()).unwrap();

        assert_eq!(serialized.title, "Unprocessable Entity");
        assert_eq!(serialized.status, StatusCode::UNPROCESSABLE_ENTITY.as_u16());
        assert_eq!(serialized.detail, "Validation failed");

        let errors = serialized.errors.unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "email");
        assert_eq!(errors[0].code, "email");
        assert_eq!(errors[0].message, "Validation failed for field 'email'");
    }

    fn validation_errors() -> ValidationErrors {
        TestPayload {
            email: "not-an-email".into(),
        }
        .validate()
        .unwrap_err()
    }
}

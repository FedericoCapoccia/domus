use axum::{
    body::Body,
    extract::rejection::JsonRejection,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct FieldError {
    field: String,
    code: String,
    message: String,
}

#[derive(Serialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    type_: String,
    title: String,
    status: u16,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<FieldError>>,
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

// TODO: Add 'WWW-Authenticate' header when 401
impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/problem+json")
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

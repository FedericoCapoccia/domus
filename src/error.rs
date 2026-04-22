use axum::{
    body::Body,
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
            type_: String::from("about:blank"),
            title,
            status: status.as_u16(),
            detail,
            errors,
        }
    }

    pub fn unauthorized(detail: String) -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            String::from("Unauthorized"),
            detail,
            None,
        )
    }

    pub fn internal_error() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("Internal Server Error"),
            String::from("An unexpected error occurred"),
            None,
        )
    }

    pub fn unprocessable_entity(detail: String, errors: Vec<FieldError>) -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            String::from("Unprocessable Entity"),
            detail,
            Some(errors),
        )
    }

    pub fn conflict(detail: String) -> Self {
        Self::new(StatusCode::CONFLICT, String::from("Conflict"), detail, None)
    }
}

impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/problem+json")
            .body(Body::from(serde_json::to_string(&self).unwrap_or_default()))
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
        ProblemDetails::unprocessable_entity(String::from("Validation failed"), errors)
    }
}

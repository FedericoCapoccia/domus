use axum::{
    Json,
    extract::{FromRequest, Request},
};
use validator::Validate;

use crate::error::ProblemDetails;

pub struct ValidatedJson<T>(pub T);
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned + Validate,
    Json<T>: FromRequest<S, Rejection = axum::extract::rejection::JsonRejection>,
{
    type Rejection = ProblemDetails;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(payload) = Json::<T>::from_request(req, state)
            .await
            .map_err(ProblemDetails::from)?;
        payload.validate().map_err(ProblemDetails::from)?;
        Ok(Self(payload))
    }
}

use hyper::{Body, Response, StatusCode};
use std::error::Error;

/// A result of an HTTP operation
pub(super) type HttpResult<T> = Result<T, Box<dyn HttpError>>;

/// A generic HTTP error trait
pub(super) trait HttpError {
    /// Returns the status code of this error
    fn status(&self) -> StatusCode;
}

impl From<Box<dyn HttpError>> for Response<Body> {
    fn from(err: Box<dyn HttpError>) -> Self {
        Response::builder()
            .status(err.status())
            .body(Body::empty())
            .unwrap()
    }
}

#[derive(Debug)]
pub struct InternalServerError;

impl HttpError for InternalServerError {
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

/// Allows to convert any kind of error to a 500 Internal Server Error using `?`
impl<E: Into<Box<dyn Error>>> From<E> for Box<dyn HttpError> {
    fn from(_err: E) -> Self {
        Box::new(InternalServerError)
    }
}

#[derive(Debug)]
pub struct BadRequest;

impl HttpError for BadRequest {
    fn status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

pub(super) fn to_http_err<E: Error, H: HttpError + 'static>(
    http_err: H,
) -> impl FnOnce(E) -> Box<dyn HttpError> {
    move |_err| -> Box<dyn HttpError> { Box::new(http_err) }
}

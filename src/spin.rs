//! Spin SDK implementations for the serverless-web traits.

use crate::{Handler, Request, Response, ResponseBuilder};
use http::{HeaderMap, HeaderValue as HttpHeaderValue, StatusCode};
use std::borrow::Cow;

#[cfg(feature = "serde")]
use crate::serde::{RequestExt, ResponseBuilderExt, ResponseExt};

/// Spin HTTP Request wrapper that implements our Request trait
pub struct SpinRequest(pub spin_sdk::http::Request);

impl Request for SpinRequest {
    fn method(&self) -> &str {
        self.0.method().as_str()
    }

    fn path(&self) -> &str {
        self.0.path()
    }

    fn header(&self, name: &str) -> Option<Cow<'_, str>> {
        self.0.header(name).and_then(|hv| {
            hv.as_str().map(Cow::Borrowed).or_else(|| {
                Some(Cow::Owned(
                    String::from_utf8_lossy(hv.as_bytes()).into_owned(),
                ))
            })
        })
    }

    fn body(&self) -> &[u8] {
        self.0.body()
    }
}

impl From<spin_sdk::http::Request> for SpinRequest {
    fn from(req: spin_sdk::http::Request) -> Self {
        SpinRequest(req)
    }
}

impl TryFrom<http::Request<Vec<u8>>> for SpinRequest {
    type Error = anyhow::Error;

    fn try_from(value: http::Request<Vec<u8>>) -> Result<Self, Self::Error> {
        let mut builder = spin_sdk::http::Request::builder()
            .method(value.method().as_str())
            .uri(value.uri().to_string());

        for (name, value) in value.headers() {
            builder = builder.header(name.as_str(), value.to_str()?);
        }

        Ok(SpinRequest(builder.body(value.into_body())?))
    }
}

/// Spin HTTP Response wrapper that implements our Response trait
pub struct SpinResponse(pub spin_sdk::http::Response);

impl Response for SpinResponse {
    fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.0.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn header(&self, name: &str) -> Option<Cow<'_, str>> {
        self.0.header(name).and_then(|hv| {
            hv.as_str().map(Cow::Borrowed).or_else(|| {
                Some(Cow::Owned(
                    String::from_utf8_lossy(hv.as_bytes()).into_owned(),
                ))
            })
        })
    }

    fn body(&self) -> &[u8] {
        self.0.body()
    }
}

impl From<SpinResponse> for spin_sdk::http::Response {
    fn from(resp: SpinResponse) -> Self {
        resp.0
    }
}

impl TryFrom<http::Response<Vec<u8>>> for SpinResponse {
    type Error = anyhow::Error;

    fn try_from(value: http::Response<Vec<u8>>) -> Result<Self, Self::Error> {
        let mut builder = spin_sdk::http::Response::builder().status(value.status().as_u16());

        for (name, value) in value.headers() {
            builder = builder.header(name.as_str(), value.to_str()?);
        }

        Ok(SpinResponse(builder.body(value.into_body())?))
    }
}

/// Spin HTTP ResponseBuilder that implements our ResponseBuilder trait
pub struct SpinResponseBuilder(pub spin_sdk::http::ResponseBuilder);

impl ResponseBuilder for SpinResponseBuilder {
    type Resp = SpinResponse;

    fn new() -> Self {
        SpinResponseBuilder(spin_sdk::http::Response::builder())
    }

    fn status(self, status: StatusCode) -> Self {
        SpinResponseBuilder(self.0.status(status.as_u16()))
    }

    fn header(self, name: &str, value: &str) -> Self {
        SpinResponseBuilder(self.0.header(name, value))
    }

    fn body(self, body: impl Into<Vec<u8>>) -> Self {
        SpinResponseBuilder(self.0.body(body.into()))
    }

    fn build(self) -> Self::Resp {
        SpinResponse(self.0.build())
    }
}

#[cfg(feature = "serde")]
impl ResponseBuilderExt for SpinResponseBuilder {}

/// Spin handler that implements our Handler trait
pub struct SpinHandler<F>(pub F);

impl<F> Handler for SpinHandler<F>
where
    F: Fn(spin_sdk::http::Request) -> spin_sdk::http::Response + Send + Sync + 'static,
{
    type Req = SpinRequest;
    type Resp = SpinResponse;

    fn handle(&self, req: Self::Req) -> Self::Resp {
        SpinResponse((self.0)(req.0))
    }
}

/// Convenience function to create a Spin handler from a function
pub fn spin_handler<F>(f: F) -> SpinHandler<F>
where
    F: Fn(spin_sdk::http::Request) -> spin_sdk::http::Response + Send + Sync + 'static,
{
    SpinHandler(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{Request, Response};

    #[test]
    fn test_request_conversion() {
        let http_req = Request::builder()
            .method("GET")
            .uri("/test")
            .header("Content-Type", "application/json")
            .body(vec![])
            .unwrap();

        let spin_req: SpinRequest = http_req.try_into().unwrap();
        assert_eq!(spin_req.method(), "GET");
        assert_eq!(spin_req.path(), "/test");
        assert_eq!(spin_req.header("Content-Type").unwrap(), "application/json");
    }

    #[test]
    fn test_response_conversion() {
        let http_resp = Response::builder()
            .status(200)
            .header("Content-Type", "text/plain")
            .body(vec![])
            .unwrap();

        let spin_resp: SpinResponse = http_resp.try_into().unwrap();
        assert_eq!(spin_resp.status(), StatusCode::OK);
        assert_eq!(spin_resp.header("Content-Type").unwrap(), "text/plain");
    }
}

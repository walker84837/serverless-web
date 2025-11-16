//! Cloudflare Workers implementations for the serverless-web traits.

use crate::{Handler, Request, Response, ResponseBuilder};
use http::{HeaderMap, HeaderValue, StatusCode};
use std::borrow::Cow;

#[cfg(feature = "serde")]
use crate::serde::{RequestExt, ResponseBuilderExt, ResponseExt};

/// Cloudflare Workers Request wrapper that implements our Request trait
pub struct CfRequest(pub worker::Request);

impl Request for CfRequest {
    fn method(&self) -> &str {
        self.0.method().as_str()
    }

    fn path(&self) -> &str {
        self.0.path()
    }

    fn header(&self, name: &str) -> Option<Cow<'_, str>> {
        self.0.headers().get(name).and_then(|value| {
            value
                .to_str()
                .map(Cow::Borrowed)
                .ok()
                .or_else(|| Some(Cow::Owned(value.to_string())))
        })
    }

    fn body(&self) -> &[u8] {
        self.0.body().as_ref().unwrap_or(&[])
    }
}

impl TryFrom<http::Request<Vec<u8>>> for CfRequest {
    type Error = worker::Error;

    fn try_from(value: http::Request<Vec<u8>>) -> Result<Self, Self::Error> {
        let mut builder = worker::Request::builder()
            .method(value.method().as_str())
            .uri(value.uri().to_string());

        for (name, value) in value.headers() {
            builder = builder.header(name.as_str(), value.to_str()?);
        }

        Ok(CfRequest(builder.body(value.into_body())?))
    }
}

impl From<worker::Request> for CfRequest {
    fn from(req: worker::Request) -> Self {
        CfRequest(req)
    }
}

/// Cloudflare Workers Response wrapper that implements our Response trait
pub struct CfResponse(pub worker::Response);

impl Response for CfResponse {
    fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn header(&self, name: &str) -> Option<Cow<'_, str>> {
        self.0.headers().get(name).and_then(|value| {
            value
                .to_str()
                .map(Cow::Borrowed)
                .ok()
                .or_else(|| Some(Cow::Owned(value.to_string())))
        })
    }

    fn body(&self) -> &[u8] {
        self.0.body().as_ref().unwrap_or(&[])
    }
}

impl TryFrom<http::Response<Vec<u8>>> for CfResponse {
    type Error = worker::Error;

    fn try_from(value: http::Response<Vec<u8>>) -> Result<Self, Self::Error> {
        let mut builder = worker::Response::builder().status(value.status().as_u16());

        for (name, value) in value.headers() {
            builder = builder.header(name.as_str(), value.to_str()?);
        }

        Ok(CfResponse(builder.body(value.into_body())?))
    }
}

impl From<CfResponse> for worker::Response {
    fn from(resp: CfResponse) -> Self {
        resp.0
    }
}

/// Cloudflare Workers ResponseBuilder that implements our ResponseBuilder trait
pub struct CfResponseBuilder(pub worker::ResponseBuilder);

impl ResponseBuilder for CfResponseBuilder {
    type Resp = CfResponse;

    fn new() -> Self {
        CfResponseBuilder(worker::Response::builder())
    }

    fn status(self, status: StatusCode) -> Self {
        CfResponseBuilder(self.0.status(status.as_u16()))
    }

    fn header(self, name: &str, value: &str) -> Self {
        CfResponseBuilder(self.0.header(name, value))
    }

    fn body(self, body: impl Into<Vec<u8>>) -> Self {
        CfResponseBuilder(self.0.body(body.into()))
    }

    fn build(self) -> Self::Resp {
        CfResponse(self.0.build())
    }
}

#[cfg(feature = "serde")]
impl ResponseBuilderExt for CfResponseBuilder {}

/// Cloudflare Workers handler that implements our Handler trait
pub struct CfHandler<F>(pub F);

impl<F> Handler for CfHandler<F>
where
    F: Fn(worker::Request) -> worker::Response + Send + Sync + 'static,
{
    type Req = CfRequest;
    type Resp = CfResponse;

    fn handle(&self, req: Self::Req) -> Self::Resp {
        CfResponse((self.0)(req.0))
    }
}

/// Convenience function to create a Cloudflare handler from a function
pub fn cf_handler<F>(f: F) -> CfHandler<F>
where
    F: Fn(worker::Request) -> worker::Response + Send + Sync + 'static,
{
    CfHandler(f)
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

        let cf_req: CfRequest = http_req.try_into().unwrap();
        assert_eq!(cf_req.method(), "GET");
        assert_eq!(cf_req.path(), "/test");
        assert_eq!(cf_req.header("Content-Type").unwrap(), "application/json");
    }

    #[test]
    fn test_response_conversion() {
        let http_resp = Response::builder()
            .status(200)
            .header("Content-Type", "text/plain")
            .body(vec![])
            .unwrap();

        let cf_resp: CfResponse = http_resp.try_into().unwrap();
        assert_eq!(cf_resp.status(), StatusCode::OK);
        assert_eq!(cf_resp.header("Content-Type").unwrap(), "text/plain");
    }
}

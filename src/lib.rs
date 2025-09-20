//! # serverless-web

#![warn(clippy::pedantic)]
#![warn(missing_docs)]

use http::StatusCode;
use std::borrow::Cow;

/// HTTP request
pub trait Request {
    /// ...
    fn method(&self) -> &str;
    /// ...
    fn path(&self) -> &str;
    /// ...
    fn header(&self, name: &str) -> Option<Cow<'_, str>>;
    /// ...
    fn body(&self) -> &[u8];
}

/// ...
pub trait Response {
    /// ...
    fn status(&self) -> StatusCode;
    /// ...
    fn header(&self, name: &str) -> Option<Cow<'_, str>>;
    /// ...
    fn body(&self) -> &[u8];
}

/// ...
pub trait ResponseBuilder {
    /// ...
    type Resp: Response;

    /// ...
    fn new() -> Self;
    /// ...
    fn status(self, status: StatusCode) -> Self;
    /// ...
    fn header(self, name: &str, value: &str) -> Self;
    /// ...
    fn body(self, body: impl Into<Vec<u8>>) -> Self;
    /// ...
    fn build(self) -> Self::Resp;
}

/// ...
pub trait Handler {
    /// ...
    type Req: Request;
    /// ...
    type Resp: Response;

    /// ...
    fn handle(&self, req: Self::Req) -> Self::Resp;
}

#[cfg(feature = "serde")]
pub trait RequestExt: Request {
    fn json_body<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(self.body())
    }
}

#[cfg(feature = "serde")]
pub trait ResponseBuilderExt: ResponseBuilder {
    fn json<T: serde::ser::Serialize>(self, value: &T) -> Result<Self::Resp, serde_json::Error> {
        let bytes = serde_json::to_vec(value)?;
        Ok(self
            .header("content-type", "application/json")
            .body(bytes)
            .build())
    }
}

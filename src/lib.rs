//! # serverless-web
//! ...
//!

#![warn(clippy::pedantic)]
#![warn(missing_docs)]

#[cfg(feature = "cf_workers")]
pub mod cloudflare;

#[cfg(feature = "spin")]
pub mod spin;

#[cfg(feature = "serde")]
pub mod serde;

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

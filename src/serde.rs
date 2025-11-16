use crate::{Request, Response, ResponseBuilder};
use serde::{de::DeserializeOwned, ser::Serialize};

/// Extension trait for `Request` to deserialize JSON bodies
pub trait RequestExt: Request {
    /// Deserialize the request body as JSON
    fn json_body<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(self.body())
    }
}

impl<T: Request> RequestExt for T {}

/// Extension trait for `Response` to deserialize JSON bodies
pub trait ResponseExt: Response {
    /// Deserialize the response body as JSON
    fn json_body<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(self.body())
    }
}

impl<T: Response> ResponseExt for T {}

/// Extension trait for `ResponseBuilder` to serialize JSON responses
pub trait ResponseBuilderExt: ResponseBuilder {
    /// Set the response body to JSON and appropriate content-type header
    fn json<T: Serialize>(self, value: &T) -> Result<Self::Resp, serde_json::Error> {
        let bytes = serde_json::to_vec(value)?;
        Ok(self
            .header("content-type", "application/json")
            .body(bytes)
            .build())
    }
}

impl<T: ResponseBuilder> ResponseBuilderExt for T {}

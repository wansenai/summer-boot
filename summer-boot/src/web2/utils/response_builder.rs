use serde::Serialize;

use crate::http_types::headers::{HeaderName, ToHeaderValues};
use crate::http_types::{Body, Mime, StatusCode};
use crate::Response;
use std::convert::TryInto;

#[derive(Debug)]

pub struct ResponseBuilder(Response);

impl ResponseBuilder {
    pub(crate) fn new<S>(status: S) -> Self
    where
        S: TryInto<StatusCode>,
        S::Error: std::fmt::Debug,
    {
        Self(Response::new(status))
    }

    pub fn build(self) -> Response {
        self.0
    }

    pub fn header(mut self, key: impl Into<HeaderName>, value: impl ToHeaderValues) -> Self {
        self.0.insert_header(key, value);
        self
    }

    pub fn content_type(mut self, content_type: impl Into<Mime>) -> Self {
        self.0.set_content_type(content_type);
        self
    }

    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.0.set_body(body);
        self
    }


    pub fn body_json(self, json: &impl Serialize) -> crate::Result<Self> {
        Ok(self.body(Body::from_json(json)?))
    }

    pub fn body_string(self, string: String) -> Self {
        self.body(Body::from_string(string))
    }

   
    pub fn body_bytes(self, bytes: impl AsRef<[u8]>) -> Self {
        self.body(Body::from(bytes.as_ref()))
    }

    pub async fn body_file(self, path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        Ok(self.body(Body::from_file(path).await?))
    }
}

impl From<ResponseBuilder> for Response {
    fn from(response_builder: ResponseBuilder) -> Response {
        response_builder.build()
    }
}
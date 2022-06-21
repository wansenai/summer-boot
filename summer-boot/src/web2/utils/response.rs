use std::convert::TryInto;
use std::fmt::{Debug, Display};
use std::ops::Index;

use serde::Serialize;

use crate::http_types::headers::{self, HeaderName, HeaderValues, ToHeaderValues};
use crate::http_types::{self, Body, Error, Mime, StatusCode};
use crate::ResponseBuilder;

/// HTTP response
#[derive(Debug)]
pub struct Response {
    pub(crate) res: http_types::Response,
    pub(crate) error: Option<Error>,
}

impl Response {
    /// 创建一个新的实例
    #[must_use]
    pub fn new<S>(status: S) -> Self
    where
        S: TryInto<StatusCode>,
        S::Error: Debug,
    {
        let res = http_types::Response::new(status);
        Self {
            res,
            error: None,
        }
    }

    #[must_use]
    pub fn builder<S>(status: S) -> ResponseBuilder
    where
        S: TryInto<StatusCode>,
        S::Error: Debug,
    {
        ResponseBuilder::new(status)
    }

    #[must_use]
    pub fn status(&self) -> crate::StatusCode {
        self.res.status()
    }

    pub fn set_status<S>(&mut self, status: S)
    where
        S: TryInto<StatusCode>,
        S::Error: Debug,
    {
        let status = status
            .try_into()
            .expect("无法转换为有效的 `StatusCode`");

        self.res.set_status(status);
    }

    #[must_use]
    pub fn len(&self) -> Option<usize> {
        self.res.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> Option<bool> {
        Some(self.res.len()? == 0)
    }

    #[must_use]
    pub fn header(&self, name: impl Into<HeaderName>) -> Option<&HeaderValues> {
        self.res.header(name)
    }

    #[must_use]
    pub fn header_mut(&mut self, name: impl Into<HeaderName>) -> Option<&mut HeaderValues> {
        self.res.header_mut(name)
    }

    pub fn remove_header(&mut self, name: impl Into<HeaderName>) -> Option<HeaderValues> {
        self.res.remove_header(name)
    }

    pub fn insert_header(&mut self, key: impl Into<HeaderName>, value: impl ToHeaderValues) {
        self.res.insert_header(key, value);
    }

    pub fn append_header(&mut self, key: impl Into<HeaderName>, value: impl ToHeaderValues) {
        self.res.append_header(key, value);
    }

    #[must_use]
    pub fn iter(&self) -> headers::Iter<'_> {
        self.res.iter()
    }

    #[must_use]
    pub fn iter_mut(&mut self) -> headers::IterMut<'_> {
        self.res.iter_mut()
    }

    #[must_use]
    pub fn header_names(&self) -> headers::Names<'_> {
        self.res.header_names()
    }

    #[must_use]
    pub fn header_values(&self) -> headers::Values<'_> {
        self.res.header_values()
    }

    #[must_use]
    pub fn content_type(&self) -> Option<Mime> {
        self.res.content_type()
    }

    pub fn set_content_type(&mut self, mime: impl Into<Mime>) {
        self.res.set_content_type(mime.into());
    }

    /// 设置body读取.
    pub fn set_body(&mut self, body: impl Into<Body>) {
        self.res.set_body(body);
    }

    pub fn take_body(&mut self) -> Body {
        self.res.take_body()
    }

    pub fn swap_body(&mut self, body: &mut Body) {
        self.res.swap_body(body)
    }

    pub fn body_json(&mut self, json: &impl Serialize) -> crate::Result<()> {
        self.res.set_body(Body::from_json(json)?);
        Ok(())
    }

    pub fn body_string(&mut self, string: String) {
        self.res.set_body(Body::from_string(string));
    }

    pub fn body_bytes(&mut self, bytes: impl AsRef<[u8]>) {
        self.set_body(Body::from(bytes.as_ref()));
    }

    pub async fn body_file(&mut self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        self.set_body(Body::from_file(path).await?);
        Ok(())
    }

    #[cfg(feature = "cookies")]
    pub fn insert_cookie(&mut self, cookie: Cookie<'static>) {
        self.cookie_events.push(CookieEvent::Added(cookie));
    }

    #[cfg(feature = "cookies")]
    pub fn remove_cookie(&mut self, cookie: Cookie<'static>) {
        self.cookie_events.push(CookieEvent::Removed(cookie));
    }

    pub fn error(&self) -> Option<&Error> {
        self.error.as_ref()
    }

    pub fn downcast_error<E>(&self) -> Option<&E>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        self.error.as_ref()?.downcast_ref()
    }

    pub fn take_error(&mut self) -> Option<Error> {
        self.error.take()
    }

    pub fn set_error(&mut self, error: impl Into<Error>) {
        self.error = Some(error.into());
    }

    #[must_use]
    pub fn ext<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.res.ext().get()
    }

    pub fn insert_ext<T: Send + Sync + 'static>(&mut self, val: T) {
        self.res.ext_mut().insert(val);
    }

    pub fn from_res<T>(value: T) -> Self
    where
        T: Into<http_types::Response>,
    {
        let res: http_types::Response = value.into();
        Self {
            res,
            error: None,
            #[cfg(feature = "cookies")]
            cookie_events: vec![],
        }
    }
}

impl AsRef<http_types::Response> for Response {
    fn as_ref(&self) -> &http_types::Response {
        &self.res
    }
}

impl AsMut<http_types::Response> for Response {
    fn as_mut(&mut self) -> &mut http_types::Response {
        &mut self.res
    }
}

impl AsRef<http_types::Headers> for Response {
    fn as_ref(&self) -> &http_types::Headers {
        self.res.as_ref()
    }
}

impl AsMut<http_types::Headers> for Response {
    fn as_mut(&mut self) -> &mut http_types::Headers {
        self.res.as_mut()
    }
}

impl From<Response> for http_types::Response {
    fn from(response: Response) -> http_types::Response {
        response.res
    }
}

impl From<http_types::Body> for Response {
    fn from(body: http_types::Body) -> Self {
        let mut res = Response::new(200);
        res.set_body(body);
        res
    }
}

impl From<serde_json::Value> for Response {
    fn from(json_value: serde_json::Value) -> Self {
        Body::from_json(&json_value)
            .map(|body| body.into())
            .unwrap_or_else(|_| Response::new(StatusCode::InternalServerError))
    }
}

impl From<Error> for Response {
    fn from(err: Error) -> Self {
        Self {
            res: http_types::Response::new(err.status()),
            error: Some(err),
            #[cfg(feature = "cookies")]
            cookie_events: vec![],
        }
    }
}

impl From<http_types::Response> for Response {
    fn from(res: http_types::Response) -> Self {
        Self {
            res,
            error: None,
            #[cfg(feature = "cookies")]
            cookie_events: vec![],
        }
    }
}

impl From<StatusCode> for Response {
    fn from(status: StatusCode) -> Self {
        let res: http_types::Response = status.into();
        res.into()
    }
}

impl From<String> for Response {
    fn from(s: String) -> Self {
        Body::from_string(s).into()
    }
}

impl<'a> From<&'a str> for Response {
    fn from(s: &'a str) -> Self {
        Body::from_string(String::from(s)).into()
    }
}

impl IntoIterator for Response {
    type Item = (HeaderName, HeaderValues);
    type IntoIter = http_types::headers::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.res.into_iter()
    }
}

impl<'a> IntoIterator for &'a Response {
    type Item = (&'a HeaderName, &'a HeaderValues);
    type IntoIter = http_types::headers::Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.res.iter()
    }
}

impl<'a> IntoIterator for &'a mut Response {
    type Item = (&'a HeaderName, &'a mut HeaderValues);
    type IntoIter = http_types::headers::IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.res.iter_mut()
    }
}

impl Index<HeaderName> for Response {
    type Output = HeaderValues;

    #[inline]
    fn index(&self, name: HeaderName) -> &HeaderValues {
        &self.res[name]
    }
}

impl Index<&str> for Response {
    type Output = HeaderValues;

    #[inline]
    fn index(&self, name: &str) -> &HeaderValues {
        &self.res[name]
    }
}
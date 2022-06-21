use async_std::io::{self, prelude::*};
use async_std::task::{Context, Poll};
use routefinder::Captures;

use std::ops::Index;
use std::pin::Pin;

use crate::http_types::format_err;
use crate::http_types::headers::{self, HeaderName, HeaderValues, ToHeaderValues};
use crate::http_types::{self, Body, Method, Mime, StatusCode, Url, Version};
use crate::Response;

pin_project_lite::pin_project! {
    /// HTTP request.
    ///
    /// 请求、路由参数以及访问请求的各种方式。
    /// 中间件和endpoints之间的通信
    #[derive(Debug)]
    pub struct Request<State> {
        pub(crate) state: State,
        #[pin]
        pub(crate) req: http_types::Request,
        pub(crate) route_params: Vec<Captures<'static, 'static>>,
    }
}

impl<State> Request<State> {
    /// 创建一个新的 `Request`.
    pub(crate) fn new(
        state: State,
        req: http_types::Request,
        route_params: Vec<Captures<'static, 'static>>,
    ) -> Self {
        Self {
            state,
            req,
            route_params,
        }
    }

    /// 访问请求的HTTP方法。
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use summer_boot::Request;
    ///
    /// let mut app = summer_boot::new();
    /// app.at("/").get(|req: Request<()>| async move {
    ///     assert_eq!(req.method(), http_types::Method::Get);
    ///     Ok("")
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    #[must_use]
    pub fn method(&self) -> Method {
        self.req.method()
    }

    /// 访问请求的完整URI方法。
    #[must_use]
    pub fn url(&self) -> &Url {
        self.req.url()
    }

    /// 访问请求的HTTP版本。
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use summer_boot::Request;
    ///
    /// let mut app = summer_boot::new();
    /// app.at("/").get(|req: Request<()>| async move {
    ///     assert_eq!(req.version(), Some(http_types::Version::Http1_1));
    ///     Ok("")
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    #[must_use]
    pub fn version(&self) -> Option<Version> {
        self.req.version()
    }

    /// 获取基础传输的socket地址
    #[must_use]
    pub fn peer_addr(&self) -> Option<&str> {
        self.req.peer_addr()
    }

    /// 获取基础传输的本地地址
    #[must_use]
    pub fn local_addr(&self) -> Option<&str> {
        self.req.local_addr()
    }

    /// 获取此请求的远程地址。
    ///
    /// 按以下优先级确定：
    /// 1. `Forwarded` head `for` key
    /// 2. 第一个 `X-Forwarded-For` header
    /// 3. 传输的对等地址
    #[must_use]
    pub fn remote(&self) -> Option<&str> {
        self.req.remote()
    }

    /// 获取此请求的目标主机。
    ///
    /// 按以下优先级确定：
    /// 1. `Forwarded` header `host` key
    /// 2. 第一个 `X-Forwarded-Host` header
    /// 3. `Host` header
    /// 4. URL域
    #[must_use]
    pub fn host(&self) -> Option<&str> {
        self.req.host()
    }

    /// 以“Mime”形式获取请求内容类型。
    ///
    /// 这将获取请求 `Content-Type` header。
    ///
    #[must_use]
    pub fn content_type(&self) -> Option<Mime> {
        self.req.content_type()
    }

    /// 获取HTTP header.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use summer_boot::Request;
    ///
    /// let mut app = summer_boot::new();
    /// app.at("/").get(|req: Request<()>| async move {
    ///     assert_eq!(req.header("X-Forwarded-For").unwrap(), "127.0.0.1");
    ///     Ok("")
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    #[must_use]
    pub fn header(
        &self,
        key: impl Into<http_types::headers::HeaderName>,
    ) -> Option<&http_types::headers::HeaderValues> {
        self.req.header(key)
    }

    /// 获取标题的可变引用。
    pub fn header_mut(&mut self, name: impl Into<HeaderName>) -> Option<&mut HeaderValues> {
        self.req.header_mut(name)
    }

    /// 设置一个 HTTP header.
    pub fn insert_header(
        &mut self,
        name: impl Into<HeaderName>,
        values: impl ToHeaderValues,
    ) -> Option<HeaderValues> {
        self.req.insert_header(name, values)
    }

    /// 将header添加到headers。
    ///
    /// 与 `insert` 不同，此函数不会重写标头的内容，而是插入
    /// 如果没有header。添加到现有的headers列表中。
    pub fn append_header(&mut self, name: impl Into<HeaderName>, values: impl ToHeaderValues) {
        self.req.append_header(name, values)
    }

    /// 移除一个 header.
    pub fn remove_header(&mut self, name: impl Into<HeaderName>) -> Option<HeaderValues> {
        self.req.remove_header(name)
    }

    /// 以任意顺序访问所有header的迭代。
    #[must_use]
    pub fn iter(&self) -> headers::Iter<'_> {
        self.req.iter()
    }

    /// 迭代器以任意顺序访问所有header，并对值进行可变引用。
    #[must_use]
    pub fn iter_mut(&mut self) -> headers::IterMut<'_> {
        self.req.iter_mut()
    }

    /// 以任意顺序访问所有header名称的迭代。
    #[must_use]
    pub fn header_names(&self) -> headers::Names<'_> {
        self.req.header_names()
    }

    /// 以任意顺序访问所有header值的迭代。
    #[must_use]
    pub fn header_values(&self) -> headers::Values<'_> {
        self.req.header_values()
    }

    /// 获取请求扩展值。
    #[must_use]
    pub fn ext<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.req.ext().get()
    }

    /// 获取对存储在请求扩展中的值的可变引用。
    #[must_use]
    pub fn ext_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.req.ext_mut().get_mut()
    }

    /// 设置请求扩展值。
    pub fn set_ext<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.req.ext_mut().insert(val)
    }

    #[must_use]
    ///  访问应用程序范围的状态。
    pub fn state(&self) -> &State {
        &self.state
    }

    /// 按名称提取和解析路由参数。
    ///
    /// 以 `&str` 形式返回参数，该参数是从此 `Request` 借用的。
    /// 
    /// 名称应不包括引用 `:`。
    ///
    /// # Errors
    ///
    /// 如果 `key` 不是路由的有效参数，则返回错误。
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use summer_boot::{Request, Result};
    ///
    /// async fn greet(req: Request<()>) -> Result<String> {
    ///     let name = req.param("name").unwrap_or("world");
    ///     Ok(format!("Hello, {}!", name))
    /// }
    ///
    /// let mut app = summer_boot::new();
    /// app.at("/hello").get(greet);
    /// app.at("/hello/:name").get(greet);
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    pub fn param(&self, key: &str) -> crate::Result<&str> {
        self.route_params
            .iter()
            .rev()
            .find_map(|captures| captures.get(key))
            .ok_or_else(|| format_err!("Param \"{}\" not found", key.to_string()))
    }

    /// 从路由中提取通配符（如果存在）
    ///
    /// 以 `&str` 形式返回参数，该参数是从此 `Request` 借用的。
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use summer_boot::{Request, Result};
    ///
    /// async fn greet(req: Request<()>) -> Result<String> {
    ///     let name = req.wildcard().unwrap_or("world");
    ///     Ok(format!("Hello, {}!", name))
    /// }
    ///
    /// let mut app = summer_boot::new();
    /// app.at("/hello/*").get(greet);
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    pub fn wildcard(&self) -> Option<&str> {
        self.route_params
            .iter()
            .rev()
            .find_map(|captures| captures.wildcard())
    }

    /// 
    /// 使用[serde_qs](https://docs.rs/serde_qs)将URL查询组件解析为结构
    /// 将整个查询作为未解析的字符串获取，使用 `request.url().query()`。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use summer_boot::http_types::{self, convert::Deserialize};
    /// use summer_boot::Request;
    ///
    /// // 所有权结构:
    ///
    /// #[derive(Deserialize)]
    /// struct Index {
    ///     page: u32,
    ///     selections: HashMap<String, String>,
    /// }
    ///
    /// let req: Request<()> = http_types::Request::get("https://baidu.com/get?page=2&selections[width]=narrow&selections[height]=tall").into();
    /// let Index { page, selections } = req.query().unwrap();
    /// assert_eq!(page, 2);
    /// assert_eq!(selections["width"], "narrow");
    /// assert_eq!(selections["height"], "tall");
    ///
    /// // 使用借用s:
    ///
    /// #[derive(Deserialize)]
    /// struct Query<'q> {
    ///     format: &'q str,
    /// }
    ///
    /// let req: Request<()> = http_types::Request::get("https://httpbin.org/get?format=bananna").into();
    /// let Query { format } = req.query().unwrap();
    /// assert_eq!(format, "bananna");
    /// ```
    pub fn query<'de, T: serde::de::Deserialize<'de>>(&'de self) -> crate::Result<T> {
        self.req.query()
    }

    /// 设置body读取
    pub fn set_body(&mut self, body: impl Into<Body>) {
        self.req.set_body(body)
    }

    /// 处理请求 `Body`
    ///
    /// 可以在获取或读取body后调用此方法，
    /// 但是将返回一个空的`Body`.
    ///
    /// 这对于通过AsyncReader或AsyncBufReader有用。
    pub fn take_body(&mut self) -> Body {
        self.req.take_body()
    }

    /// 将整个请求body读取字节缓冲区。
    ///
    /// 可以在读取body后调用此方法，但生成空缓冲区。
    ///
    /// # Errors
    ///
    /// 读取body时遇到的任何I/O错误都会立即返回错误 `Err`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use summer_boot::Request;
    ///
    /// let mut app = summer_boot::new();
    /// app.at("/").get(|mut req: Request<()>| async move {
    ///     let _body: Vec<u8> = req.body_bytes().await.unwrap();
    ///     Ok("")
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    pub async fn body_bytes(&mut self) -> crate::Result<Vec<u8>> {
        let res = self.req.body_bytes().await?;
        Ok(res)
    }

    /// 将整个请求body读取字符串。
    ///
    /// 可以在读取body后调用此方法，但生成空缓冲区。
    ///
    /// # Errors
    /// 
    /// 读取body时遇到的任何I/O错误都会立即返回错误 `Err`
    ///
    /// 如果body不能解释有效的UTF-8，则返回 `Err`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use summer_boot::Request;
    ///
    /// let mut app = summer_boot::new();
    /// app.at("/").get(|mut req: Request<()>| async move {
    ///     let _body: String = req.body_string().await.unwrap();
    ///     Ok("")
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    pub async fn body_string(&mut self) -> crate::Result<String> {
        let res = self.req.body_string().await?;
        Ok(res)
    }

    /// 通过json读取并反序列化整个请求body。
    ///
    /// # Errors
    ///
    /// 读取body时遇到的任何I/O错误都会立即返回错误 `Err`
    ///
    /// 如果无法将body解释为目标类型 `T` 的有效json，则返回 `Err`
    pub async fn body_json<T: serde::de::DeserializeOwned>(&mut self) -> crate::Result<T> {
        let res = self.req.body_json().await?;
        Ok(res)
    }

    /// 将请求主体解析为表单
    ///
    /// ```rust
    /// use serde::Deserialize;
    /// # fn main() -> Result<(), std::io::Error> { async_std::task::block_on(async {
    /// let mut app = summer_boot::new();
    ///
    /// #[derive(Deserialize)]
    /// struct Animal {
    ///   name: String,
    ///   legs: u8
    /// }
    ///
    /// app.at("/").post(|mut req: summer_boot::Request<()>| async move {
    ///     let animal: Animal = req.body_form().await?;
    ///     Ok(format!(
    ///         "hello, {}! i've put in an order for {} shoes",
    ///         animal.name, animal.legs
    ///     ))
    /// });
    ///
    /// # if false {
    /// app.listen("localhost:8000").await?;
    /// # }
    ///
    /// // $ curl localhost:8000/test/api -d "name=chashu&legs=4"
    /// // hello, chashu! i've put in an order for 4 shoes
    ///
    /// // $ curl localhost:8000/test/api -d "name=mary%20millipede&legs=750"
    /// // number too large to fit in target type
    /// # Ok(()) })}
    /// ```
    pub async fn body_form<T: serde::de::DeserializeOwned>(&mut self) -> crate::Result<T> {
        let res = self.req.body_form().await?;
        Ok(res)
    }

    /// 按Cookie的名称返回 `Cookie`
    #[cfg(feature = "cookies")]
    #[must_use]
    pub fn cookie(&self, name: &str) -> Option<Cookie<'static>> {
        self.ext::<CookieData>()
            .and_then(|cookie_data| cookie_data.content.read().unwrap().get(name).cloned())
    }

    /// 检索对当前session的引用。
    ///
    /// # Panics
    ///
    /// 如果summer_boot::sessions:SessionMiddleware 没有在运行。
    #[cfg(feature = "sessions")]
    pub fn session(&self) -> &crate::sessions::Session {
        self.ext::<crate::sessions::Session>().expect(
            "请求会话未初始化, 是否启用了summer_boot::sessions::SessionMiddleware?",
        )
    }

    /// 检索对当前会话的可变引用。
    ///
    /// # Panics
    ///
    /// 如果summer_boot::sessions:SessionMiddleware 没有在运行。
    #[cfg(feature = "sessions")]
    pub fn session_mut(&mut self) -> &mut crate::sessions::Session {
        self.ext_mut().expect(
            "请求会话未初始化, 是否启用了summer_boot::sessions::SessionMiddleware?",
        )
    }

    /// 获取body流的长度（如果已设置）。
    ///
    /// 将固定大小的对象传递到作为body时，会设置此值(比如字符串)。 或者缓冲区。
    /// 此API的使用应检查此值，决定是否使用 `Chunked`
    /// 设置响应长度
    #[must_use]
    pub fn len(&self) -> Option<usize> {
        self.req.len()
    }

    /// 如果请求的设置body流长度为零，则返回 `true`，否则返回 `false`。
    #[must_use]
    pub fn is_empty(&self) -> Option<bool> {
        Some(self.req.len()? == 0)
    }
}

impl<State> AsRef<http_types::Request> for Request<State> {
    fn as_ref(&self) -> &http_types::Request {
        &self.req
    }
}

impl<State> AsMut<http_types::Request> for Request<State> {
    fn as_mut(&mut self) -> &mut http_types::Request {
        &mut self.req
    }
}

impl<State> AsRef<http_types::Headers> for Request<State> {
    fn as_ref(&self) -> &http_types::Headers {
        self.req.as_ref()
    }
}

impl<State> AsMut<http_types::Headers> for Request<State> {
    fn as_mut(&mut self) -> &mut http_types::Headers {
        self.req.as_mut()
    }
}

impl<State> Read for Request<State> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.project().req.poll_read(cx, buf)
    }
}

impl<State> From<Request<State>> for http_types::Request {
    fn from(request: Request<State>) -> http_types::Request {
        request.req
    }
}

impl<State: Default> From<http_types::Request> for Request<State> {
    fn from(request: http_types::Request) -> Request<State> {
        Request::new(State::default(), request, vec![])
    }
}

impl<State: Clone + Send + Sync + 'static> From<Request<State>> for Response {
    fn from(mut request: Request<State>) -> Response {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(request.take_body());
        res
    }
}

impl<State> IntoIterator for Request<State> {
    type Item = (HeaderName, HeaderValues);
    type IntoIter = http_types::headers::IntoIter;

    /// 返回对其余项的引用的迭代.
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.req.into_iter()
    }
}

impl<'a, State> IntoIterator for &'a Request<State> {
    type Item = (&'a HeaderName, &'a HeaderValues);
    type IntoIter = http_types::headers::Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.req.iter()
    }
}

impl<'a, State> IntoIterator for &'a mut Request<State> {
    type Item = (&'a HeaderName, &'a mut HeaderValues);
    type IntoIter = http_types::headers::IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.req.iter_mut()
    }
}

impl<State> Index<HeaderName> for Request<State> {
    type Output = HeaderValues;

    /// 返回对与提供的名称相对应的值的引用。
    ///
    /// # Panics
    ///
    /// 如果 `Request` 中没有该名称，则会panic
    #[inline]
    fn index(&self, name: HeaderName) -> &HeaderValues {
        &self.req[name]
    }
}

impl<State> Index<&str> for Request<State> {
    type Output = HeaderValues;

    /// 返回对与提供的名称相对应的值的引用。
    ///
    /// # Panics
    ///
    /// 如果 `Request` 中没有该名称，则会panic
    #[inline]
    fn index(&self, name: &str) -> &HeaderValues {
        &self.req[name]
    }
}
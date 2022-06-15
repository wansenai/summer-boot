//! HTTP server
use crate::tcp;
use crate::log;
use crate::gateway;
use crate::utils;
use crate::{Endpoint, Request, Route};

use async_std::io;
use async_std::sync::Arc;

use tcp::{Listener, ToListener};
use utils::middleware::{Middleware, Next};
use utils::util;
use gateway::router::{Router, Selection};

/// HTTP服务器。
///
/// 服务器由 *state*, *endpoints* 和 *middleware* 组成。
///
/// - 服务器状态是用户定义的，通过 [`summer_boot::Server::with_state`] 函数使用. 这个
/// 状态可以用于所有应用 endpoints 共享引用.
///
/// - Endpoints 提供与指定URL [`summer_boot::Server::at`] 创建一个 *路由* 
/// 然后可以用于绑定注册到 endpoints 
/// 对于指定HTTP请求类型进行使用
///
/// - Middleware 通过附加request或
/// response 处理, 例如压缩、默认请求头或日志记录。到
/// 中间件添加到应用程序中，使用 [`summer_boot::Server::middleware`] 方法.
pub struct Server<State> {
    router: Arc<Router<State>>,
    state: State,
    /// 保存 middleware 堆栈 这里用了多线程引用计数.
    ///
    /// Vec允许我们在运行时添加中间件。
    /// 内部 Arc-s 允许在内部克隆 MiddlewareEndpoint-s 。
    /// 在这里不在Vec使用互斥体，因为在执行期间添加中间件应该是一个错误。
    #[allow(clippy::rc_buffer)]
    middleware: Arc<Vec<Arc<dyn Middleware<State>>>>,
}

impl Server<()> {
    /// 创建一个summer boot web2 server.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// let mut app = summer_boot::new();
    /// app.at("/").get(|_| async { Ok("Hello, world!") });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) }) }
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl Default for Server<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State> Server<State>
where
    State: Clone + Send + Sync + 'static,
{
    /// 创建一个可以共享应用程序作用域状态到新服务.
    ///
    // /应用程序范围的状态对于存储有用
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use summer_boot::Request;
    ///
    /// /// 共享应用程序状态
    /// #[derive(Clone)]
    /// struct State {
    ///     name: String,
    /// }
    ///
    /// // 定义状态新的一个实例
    /// let state = State {
    ///     name: "James".to_string()
    /// };
    ///
    /// // 使用状态初始化应用程序
    /// let mut app = summer_boot::with_state(state);
    /// app.at("/").get(|req: Request<State>| async move {
    ///     Ok(format!("Hello, {}!", &req.state().name))
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) }) }
    /// ```
    pub fn with_state(state: State) -> Self {
        Self {
            router: Arc::new(Router::new()),
            middleware: Arc::new(vec![
                // 暂时没有使用到cookies
                #[cfg(feature = "cookies")]
                Arc::new(cookies::CookiesMiddleware::new()),
                #[cfg(feature = "logger")]
                Arc::new(log::LogMiddleware::new()),
            ]),
            state,
        }
    }

    /// 在给定的 `path`（相对于根）处添加新路由。
    ///
    /// 路由意味着将HTTP请求映射到endpoints。
    /// 一种“目录”方法，可以方便地查看总体
    /// 应用程序结构。Endpoints仅由path和HTTP方法选择
    /// 请求：路径决定资源和HTTP请求所选资源的各个endpoints。
    /// 
    /// #Example:
    ///
    /// ```rust,no_run
    /// # let mut app = summer_boot::new();
    /// app.at("/").get(|_| async { Ok("Hello, world!") });
    /// ```
    ///
    /// 路径由零个或多个段组成，即非空字符串，由 '/' 分隔。
    ///
    /// 或者可以使用通配符
    /// `*path` 代表使用通配符配置路由
    /// 以下是一些基于HTTP的endpoints 路由选择的示例：
    ///
    /// ```rust,no_run
    /// # let mut app = summer_boot::new();
    /// app.at("/");
    /// app.at("/hello");
    /// app.at("add_two/:num");
    /// app.at("files/:user/*");
    /// app.at("static/*path");
    /// app.at("static/:context/:");
    /// ```
    ///
    /// 没有备用路由匹配，即资源已满
    /// 匹配和没有匹配，意味着添加资源的顺序没有
    pub fn at<'a>(&'a mut self, path: &str) -> Route<'a, State> {
        let router = Arc::get_mut(&mut self.router)
            .expect("服务器启动后无法注册路由");
        Route::new(router, path.to_owned())
    }

    /// 向应用程序添加中间件。
    ///
    /// 中间件提供请求/响应
    /// 日志记录或标题修改。中间件在处理请求时被调用，并且可以
    /// 继续处理（可能修改响应）或立即返回响应
    /// 响应。有关详细信息，请参考 [`Middleware`] trait
    ///
    /// 中间件只能在应用程序的 `顶层` 添加，并使用应用顺序
    pub fn with<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware<State>,
    {
        log::trace!("Adding middleware {}", middleware.name());
        let m = Arc::get_mut(&mut self.middleware)
            .expect("Registering middleware is not possible after the Server has started");
        m.push(Arc::new(middleware));
        self
    }

    /// 使用提供的侦听器异步为应用程序提供服务。
    ///
    /// 这是调用 `summer_boot::Server::bind`, 记录`ListenInfo` 实例
    /// 通过实例 `Listener::info`, 然后调用 `Listener::accept`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// let mut app = summer_boot::new();
    /// app.at("/").get(|_| async { Ok("Hello, world!") });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) }) }
    /// ```
    pub async fn listen<L: ToListener<State>>(self, listener: L) -> io::Result<()> {
        let mut listener = listener.to_listener()?;
        listener.bind(self).await?;
        for info in listener.info().iter() {
            log::info!("Server listening on {}", info);
        }
        listener.accept().await?;
        Ok(())
    }

    /// 开发中 todo
    /// 
    /// 异步绑定侦听器。
    /// 
    /// 绑定侦听器。这将打开网络端口，但没有接受传入的连接。
    /// 应调用 `Listener::listen` 开始连接
    ///
    /// 调用 `Listener::info` 的时候可能出现多个 `ListenInfo` 实例返回
    /// 这在使用例如 `ConcurrentListener` 时很有用
    /// 因为它可以让单个服务器能够侦听多个端口。
    ///
    /// # Examples
    ///
    pub async fn bind<L: ToListener<State>>(
        self,
        listener: L,
    ) -> io::Result<<L as ToListener<State>>::Listener> {
        let mut listener = listener.to_listener()?;
        listener.bind(self).await?;
        Ok(listener)
    }

    /// 响应 `Request`
    ///
    /// 此方法对于直接测试endpoints
    /// 或者通过自定义传输创建服务器。
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[async_std::main]
    /// # async fn main() -> http_types::Result<()> {
    /// #
    /// use summer_boot::http::{Url, Method, Request, Response};
    ///
    /// let mut app = summer_boot::new();
    /// app.at("/").get(|_| async { Ok("hello world") });
    ///
    /// let req = Request::new(Method::Get, Url::parse("https://example.com")?);
    /// let res: Response = app.respond(req).await?;
    ///
    /// assert_eq!(res.status(), 200);
    /// #
    /// # Ok(()) }
    /// ```
    pub async fn respond<Req, Res>(&self, req: Req) -> http_types::Result<Res>
    where
        Req: Into<http_types::Request>,
        Res: From<http_types::Response>,
    {
        let req = req.into();
        let Self {
            router,
            state,
            middleware,
        } = self.clone();

        let method = req.method().to_owned();
        let Selection { endpoint, params } = router.route(&req.url().path(), method);
        let route_params = vec![params];
        let req = Request::new(state, req, route_params);

        let next = Next {
            endpoint,
            next_middleware: &middleware,
        };

        let res = next.run(req).await;
        let res: http_types::Response = res.into();
        Ok(res.into())
    }

    /// 获取对服务器状态的引用。用于测试和嵌套：
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[derive(Clone)] struct SomeAppState;
    /// let mut app = summer_boot::with_state(SomeAppState);
    /// let mut admin = summer_boot::with_state(app.state().clone());
    /// admin.at("/").get(|_| async { Ok("nested app with cloned state") });
    /// app.at("/").nest(admin);
    /// ```
    pub fn state(&self) -> &State {
        &self.state
    }
}

impl<State: Send + Sync + 'static> std::fmt::Debug for Server<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server").finish()
    }
}

impl<State: Clone> Clone for Server<State> {
    fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
            state: self.state.clone(),
            middleware: self.middleware.clone(),
        }
    }
}

#[async_trait::async_trait]
impl<State: Clone + Sync + Send + 'static, InnerState: Clone + Sync + Send + 'static>
    Endpoint<State> for Server<InnerState>
{
    async fn call(&self, req: Request<State>) -> crate::Result {
        let Request {
            req,
            mut route_params,
            ..
        } = req;
        let path = req.url().path().to_owned();
        let method = req.method().to_owned();
        let router = self.router.clone();
        let middleware = self.middleware.clone();
        let state = self.state.clone();

        let Selection { endpoint, params } = router.route(&path, method);
        route_params.push(params);
        let req = Request::new(state, req, route_params);

        let next = Next {
            endpoint,
            next_middleware: &middleware,
        };

        Ok(next.run(req).await)
    }
}

#[util::async_trait]
impl<State: Clone + Send + Sync + Unpin + 'static> http_client::HttpClient for Server<State> {
    async fn send(&self, req: crate::http::Request) -> crate::http::Result<crate::http::Response> {
        self.respond(req).await
    }
}

#[cfg(test)]
mod test {
    use crate as summer_boot;

    #[test]
    fn allow_nested_server_with_same_state() {
        let inner = summer_boot::new();
        let mut outer = summer_boot::new();
        outer.at("/foo").get(inner);
    }

    #[test]
    fn allow_nested_server_with_different_state() {
        let inner = summer_boot::with_state(1);
        let mut outer = summer_boot::new();
        outer.at("/foo").get(inner);
    }
}

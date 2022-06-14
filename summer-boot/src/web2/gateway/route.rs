use crate::aop;
use crate::context;
use crate::log;
use crate::gateway;
use crate::utils;

use std::fmt::Debug;
use std::io;
use std::path::Path;
use std::sync::Arc;

use aop::endpoint::{Endpoint, MiddlewareEndpoint};
use utils::middleware::Middleware;
use context::serve_dir::ServeDir;
use context::serve_file::ServeFile;

use gateway::router::Router;

/// A handle to route
///
/// 所有HTTP请求都是针对资源请求的。
/// 使用`Server::at` 或者 `Route::at` 创建路由，可以使用 `Route` 类型
/// 为路径的一些HTTP方法创建endpoints 
///
#[allow(missing_debug_implementations)]
pub struct Route<'a, State> {
    router: &'a mut Router<State>,
    path: String,
    middleware: Vec<Arc<dyn Middleware<State>>>,
    /// 是否将当前路由的路径作为前缀
    /// [`strip_prefix`].
    ///
    /// [`strip_prefix`]: #method.strip_prefix
    prefix: bool,
}

impl<'a, State: Clone + Send + Sync + 'static> Route<'a, State> {
    pub(crate) fn new(router: &'a mut Router<State>, path: String) -> Route<'a, State> {
        Route {
            router,
            path,
            middleware: Vec::new(),
            prefix: false,
        }
    }

    /// 使用指定 `path` 添加路由。
    pub fn at<'b>(&'b mut self, path: &str) -> Route<'b, State> {
        let mut p = self.path.clone();

        if !p.ends_with('/') && !path.starts_with('/') {
            p.push('/');
        }

        if path != "/" {
            p.push_str(path);
        }

        Route {
            router: self.router,
            path: p,
            middleware: self.middleware.clone(),
            prefix: false,
        }
    }

    /// 获取当前路径
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 将当前路径视为前缀，并从请求中去除前缀。
    /// 这个方法标记为不稳定 unstable，后面需要summer boot 宏增强。
    /// 给endpoints提供前缀已经删除的路径。
    #[cfg(any(feature = "unstable", feature = "docs"))]
    #[cfg_attr(feature = "docs", doc(cfg(unstable)))]
    pub fn strip_prefix(&mut self) -> &mut Self {
        self.prefix = true;
        self
    }

    /// 将给定中间件作为当前路由。
    pub fn with<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware<State>,
    {
        log::trace!(
            "Adding middleware {} to route {:?}",
            middleware.name(),
            self.path
        );
        self.middleware.push(Arc::new(middleware));
        self
    }

    /// 重置当前路由的中间件
    pub fn reset_middleware(&mut self) -> &mut Self {
        self.middleware.clear();
        self
    }

    /// 在当前路径上嵌套 [`Server`]。
    ///
    /// # Note
    ///
    /// 其他服务 *始终* 具有优先权
    /// 重叠路径，这个例子输入 `/hello` 将
    /// 返回 "Unexpected" 给客户端
    ///
    /// ```no_run
    /// #[async_std::main]
    /// async fn main() -> Result<(), std::io::Error> {
    ///     let mut app = summer_boot::new();
    ///     app.at("/hello").nest({
    ///         let mut example = summer_boot::with_state("world");
    ///         example
    ///             .at("/")
    ///             .get(|req: summer_boot::Request<&'static str>| async move {
    ///                 Ok(format!("Hello {state}!", state = req.state()))
    ///             });
    ///         example
    ///     });
    ///     app.at("/*").get(|_| async { Ok("Unexpected") });
    ///     app.listen("127.0.0.1:8080").await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// [`Server`]: struct.Server.html
    pub fn nest<InnerState>(&mut self, service: crate::Server<InnerState>) -> &mut Self
    where
        State: Clone + Send + Sync + 'static,
        InnerState: Clone + Send + Sync + 'static,
    {
        let prefix = self.prefix;

        self.prefix = true;
        self.all(service);
        self.prefix = prefix;

        self
    }

    /// 静态目录服务。
    ///
    /// 每一个文件都将从磁盘io流传输，并确定了mime类型
    ///
    /// # Security
    ///
    /// 这个方法确保了除了指定文件夹下之外的文件的路径
    /// 无论是否存在都会返回StatusCode::Forbidden
    ///
    /// # Examples
    ///
    /// 本地服务提供目录 `./public/images/*` 来自路径
    /// `localhost:8080/images/*`.
    ///
    /// ```no_run
    /// #[async_std::main]
    /// async fn main() -> Result<(), std::io::Error> {
    ///     let mut app = tide::new();
    ///     app.at("/images/*").serve_dir("public/images/")?;
    ///     app.listen("127.0.0.1:8080").await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn serve_dir(&mut self, dir: impl AsRef<Path>) -> io::Result<()> {
        // 验证路径是否存在，如果不存在，则返回错误。
        let dir = dir.as_ref().to_owned().canonicalize()?;
        let prefix = self.path().to_string();
        self.get(ServeDir::new(prefix, dir));
        Ok(())
    }

    /// 提供静态文件。
    ///
    /// 每一个文件都将从磁盘io流传输，并确定了mime类型
    /// 基于magic bytes。类似serve_dir
    pub fn serve_file(&mut self, file: impl AsRef<Path>) -> io::Result<()> {
        self.get(ServeFile::init(file)?);
        Ok(())
    }

    /// 给定HTTP方法添加endpoint
    pub fn method(&mut self, method: http_types::Method, ep: impl Endpoint<State>) -> &mut Self {
        if self.prefix {
            let ep = StripPrefixEndpoint::new(ep);
            let wildcard = self.at("*");
            wildcard.router.add(
                &wildcard.path,
                method,
                MiddlewareEndpoint::wrap_with_middleware(ep, &wildcard.middleware),
            );
        } else {
            self.router.add(
                &self.path,
                method,
                MiddlewareEndpoint::wrap_with_middleware(ep, &self.middleware),
            );
        }
        self
    }

    /// 为所有HTTP方法添加一个endpoin，作为回调。
    ///
    /// 尝试使用特定HTTP方法的路由。
    pub fn all(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        if self.prefix {
            let ep = StripPrefixEndpoint::new(ep);
            let wildcard = self.at("*");
            wildcard.router.add_all(
                &wildcard.path,
                MiddlewareEndpoint::wrap_with_middleware(ep, &wildcard.middleware),
            );
        } else {
            self.router.add_all(
                &self.path,
                MiddlewareEndpoint::wrap_with_middleware(ep, &self.middleware),
            );
        }
        self
    }

    /// 为 `GET` 请求添加endpoint
    pub fn get(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Get, ep);
        self
    }

    /// 为 `HEAD` 请求添加endpoint
    pub fn head(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Head, ep);
        self
    }

    /// 为 `PUT` 请求添加endpoint
    pub fn put(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Put, ep);
        self
    }

    /// 为 `POST` 请求添加endpoint
    pub fn post(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Post, ep);
        self
    }

    /// 为 `DELETE 请求添加endpoint
    pub fn delete(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Delete, ep);
        self
    }

    /// 为 `OPTIONS` 请求添加endpoint
    pub fn options(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Options, ep);
        self
    }

    /// 为 `CONNECT` 请求添加endpoint
    pub fn connect(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Connect, ep);
        self
    }

    /// 为 `PATCH` 请求添加endpoint
    pub fn patch(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Patch, ep);
        self
    }

    /// 为 `TRACE` 请求添加endpoint
    pub fn trace(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Trace, ep);
        self
    }
}

#[derive(Debug)]
struct StripPrefixEndpoint<E>(std::sync::Arc<E>);

impl<E> StripPrefixEndpoint<E> {
    fn new(ep: E) -> Self {
        Self(std::sync::Arc::new(ep))
    }
}

impl<E> Clone for StripPrefixEndpoint<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait::async_trait]
impl<State, E> Endpoint<State> for StripPrefixEndpoint<E>
where
    State: Clone + Send + Sync + 'static,
    E: Endpoint<State>,
{
    async fn call(&self, req: crate::Request<State>) -> crate::Result {
        let crate::Request {
            state,
            mut req,
            route_params,
        } = req;

        let rest = route_params
            .iter()
            .rev()
            .find_map(|captures| captures.wildcard())
            .unwrap_or_default();

        req.url_mut().set_path(rest);

        self.0
            .call(crate::Request {
                state,
                req,
                route_params,
            })
            .await
    }
}
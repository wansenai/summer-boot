use crate::{Middleware, Request, Response};
use crate::utils;

use async_std::future::Future;
use async_std::sync::Arc;
use async_trait::async_trait;
use http_types::Result;

use utils::middleware::Next;

/// HTTP请求处理。
///
/// 这个特效是为了 `Fn` 类型自动实现的，所以很少实现，由开发者提供
///
/// 实际上 endpoint是用`Request<State>`作为参数的函数，然后将实现的类型`T`（泛型）返回`Into<Response>`
///
/// # Examples
///
/// 这里利用的异步函数，但是只有Nightly版本才可以使用，如果要使用的话就需要启用Nightly版本
/// 这个例子对`GET`请求调用返回`String`
///
/// ```no_run
/// async fn hello(_req: summer_boot::Request<()>) -> summer_boot::Result<String> {
///     Ok(String::from("hello"))
/// }
///
/// let mut app = summer_boot::new();
/// app.at("/hello").get(hello);
/// ```
///
/// 如果不使用async异步的话，例子如下：
///
/// ```no_run
/// use core::future::Future;
/// fn hello(_req: summer_boot::Request<()>) -> impl Future<Output = summer_boot::Result<String>> {
///     async_std::future::ready(Ok(String::from("hello")))
/// }
///
/// let mut app = summer_boot::new();
/// app.at("/hello").get(hello);
/// ```
///
/// summer_boot也可以使用带有`Fn`的endpoint，但是一般建议用async异步
#[async_trait]
pub trait Endpoint<State: Clone + Send + Sync + 'static>: Send + Sync + 'static {
    /// 上下文中调用endpoint
    async fn call(&self, req: Request<State>) -> crate::Result;
}

pub(crate) type DynEndpoint<State> = dyn Endpoint<State>;

#[async_trait]
impl<State, F, Fut, Res> Endpoint<State> for F
where
    State: Clone + Send + Sync + 'static,
    F: Send + Sync + 'static + Fn(Request<State>) -> Fut,
    Fut: Future<Output = Result<Res>> + Send + 'static,
    Res: Into<Response> + 'static,
{
    async fn call(&self, req: Request<State>) -> crate::Result {
        let fut = (self)(req);
        let res = fut.await?;
        Ok(res.into())
    }
}

pub(crate) struct MiddlewareEndpoint<E, State> {
    endpoint: E,
    middleware: Vec<Arc<dyn Middleware<State>>>,
}

impl<E: Clone, State> Clone for MiddlewareEndpoint<E, State> {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            middleware: self.middleware.clone(),
        }
    }
}

impl<E, State> std::fmt::Debug for MiddlewareEndpoint<E, State> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            fmt,
            "MiddlewareEndpoint (length: {})",
            self.middleware.len(),
        )
    }
}

impl<E, State> MiddlewareEndpoint<E, State>
where
    State: Clone + Send + Sync + 'static,
    E: Endpoint<State>,
{
    pub(crate) fn wrap_with_middleware(
        ep: E,
        middleware: &[Arc<dyn Middleware<State>>],
    ) -> Box<dyn Endpoint<State> + Send + Sync + 'static> {
        if middleware.is_empty() {
            Box::new(ep)
        } else {
            Box::new(Self {
                endpoint: ep,
                middleware: middleware.to_vec(),
            })
        }
    }
}

#[async_trait]
impl<E, State> Endpoint<State> for MiddlewareEndpoint<E, State>
where
    State: Clone + Send + Sync + 'static,
    E: Endpoint<State>,
{
    async fn call(&self, req: Request<State>) -> crate::Result {
        let next = Next {
            endpoint: &self.endpoint,
            next_middleware: &self.middleware,
        };
        Ok(next.run(req).await)
    }
}

#[async_trait]
impl<State: Clone + Send + Sync + 'static> Endpoint<State> for Box<dyn Endpoint<State>> {
    async fn call(&self, request: Request<State>) -> crate::Result {
        self.as_ref().call(request).await
    }
}
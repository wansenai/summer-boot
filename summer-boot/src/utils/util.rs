//! 其他util

use crate::{Middleware, Next, Request, Response};
pub use async_trait::async_trait;
use std::future::Future;

/// 定义对传入请求进行操作的中间件。
///
/// 用于定义内联中间件的闭包。
///
/// # Examples
///
/// ```rust
/// use summer_boot::utils::util;
/// use summer_boot::Request;
/// use std::time::Instant;
///
/// let mut app = summer_boot::new();
/// app.with(util::Before(|mut request: Request<()>| async move {
///     request.set_ext(Instant::now());
///     request
/// }));
/// ```
#[derive(Debug)]
pub struct Before<F>(pub F);

#[async_trait]
impl<State, F, Fut> Middleware<State> for Before<F>
where
    State: Clone + Send + Sync + 'static,
    F: Fn(Request<State>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Request<State>> + Send + Sync + 'static,
{
    async fn handle(&self, request: Request<State>, next: Next<'_, State>) -> crate::Result {
        let request = (self.0)(request).await;
        Ok(next.run(request).await)
    }
}

/// 定义对传出响应进行操作的中间件。
///
/// 用于定义内联中间件的闭包。
///
#[derive(Debug)]
pub struct After<F>(pub F);
#[async_trait]
impl<State, F, Fut> Middleware<State> for After<F>
where
    State: Clone + Send + Sync + 'static,
    F: Fn(Response) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = crate::Result> + Send + Sync + 'static,
{
    async fn handle(&self, request: Request<State>, next: Next<'_, State>) -> crate::Result {
        let response = next.run(request).await;
        (self.0)(response).await
    }
}

use crate::aop;
use crate::{Request, Response};

use std::sync::Arc;
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use aop::endpoint::DynEndpoint;

/// 异步中间件trait
#[async_trait]
pub trait Middleware<State>: Send + Sync + 'static {
    /// 异步处理请求并返回响应。
    async fn handle(&self, request: Request<State>, next: Next<'_, State>) -> crate::Result;

    /// 设置中间件的名称。默认情况下，使用类型名字.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

#[async_trait]
impl<State, F> Middleware<State> for F
where
    State: Clone + Send + Sync + 'static,
    F: Send
        + Sync
        + 'static
        + for<'a> Fn(
            Request<State>,
            Next<'a, State>,
        ) -> Pin<Box<dyn Future<Output = crate::Result> + 'a + Send>>,
{
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> crate::Result {
        (self)(req, next).await
    }
}

/// 中间件链系列其余部分，包括endpoints。
#[allow(missing_debug_implementations)]
pub struct Next<'a, State> {
    pub(crate) endpoint: &'a DynEndpoint<State>,
    pub(crate) next_middleware: &'a [Arc<dyn Middleware<State>>],
}

impl<State: Clone + Send + Sync + 'static> Next<'_, State> {
    /// 异步执行其余的中间件。
    pub async fn run(mut self, req: Request<State>) -> Response {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            match current.handle(req, self).await {
                Ok(request) => request,
                Err(err) => err.into(),
            }
        } else {
            match self.endpoint.call(req).await {
                Ok(request) => request,
                Err(err) => err.into(),
            }
        }
    }
}
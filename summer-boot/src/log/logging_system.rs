use crate::log;
use crate::{Middleware, Next, Request};

/// 记录所有传入的请求和响应
///
/// 此中间件在Summer Boot中默认启用
///
/// # Examples
///
/// ```
/// let mut app = summer_boot::new();
/// app.with(summer_boot::log::LoggingSystem::new());
/// ```
#[derive(Debug, Default, Clone)]
pub struct LoggingSystem {
    _priv: (),
}

struct LoggingSystemHasBeenRun;

impl LoggingSystem {
    /// Create a new instance of `LogMiddleware`.
    #[must_use]
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Log a request and a response.
    async fn log<'a, State: Clone + Send + Sync + 'static>(
        &'a self,
        mut req: Request<State>,
        next: Next<'a, State>,
    ) -> crate::Result {
        if req.ext::<LoggingSystemHasBeenRun>().is_some() {
            return Ok(next.run(req).await);
        }
        req.set_ext(LoggingSystemHasBeenRun);

        let path = req.url().path().to_owned();
        let method = req.method().to_string();
        log::info!("<-- Request received", {
            method: method,
            path: path,
        });
        let start = std::time::Instant::now();
        let response = next.run(req).await;
        let status = response.status();
        if status.is_server_error() {
            if let Some(error) = response.error() {
                log::error!("Internal error --> Response sent", {
                    message: format!("{:?}", error),
                    error_type: error.type_name(),
                    method: method,
                    path: path,
                    status: format!("{} - {}", status as u16, status.canonical_reason()),
                    duration: format!("{:?}", start.elapsed()),
                });
            } else {
                log::error!("Internal error --> Response sent", {
                    method: method,
                    path: path,
                    status: format!("{} - {}", status as u16, status.canonical_reason()),
                    duration: format!("{:?}", start.elapsed()),
                });
            }
        } else if status.is_client_error() {
            if let Some(error) = response.error() {
                log::warn!("Client error --> Response sent", {
                    message: format!("{:?}", error),
                    error_type: error.type_name(),
                    method: method,
                    path: path,
                    status: format!("{} - {}", status as u16, status.canonical_reason()),
                    duration: format!("{:?}", start.elapsed()),
                });
            } else {
                log::warn!("Client error --> Response sent", {
                    method: method,
                    path: path,
                    status: format!("{} - {}", status as u16, status.canonical_reason()),
                    duration: format!("{:?}", start.elapsed()),
                });
            }
        } else {
            log::info!("--> Response sent", {
                method: method,
                path: path,
                status: format!("{} - {}", status as u16, status.canonical_reason()),
                duration: format!("{:?}", start.elapsed()),
            });
        }
        Ok(response)
    }
}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for LoggingSystem {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> crate::Result {
        self.log(req, next).await
    }
}

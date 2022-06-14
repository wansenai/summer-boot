//! 表示HTTP传输和绑定的类型
use crate::Server;

mod concurrent_listener;
mod failover_listener;
mod to_listener;
mod to_listener_impls;
mod parsed_listener;
mod tcp_listener;
mod unix_listener;

use std::fmt::{Debug, Display};

use async_std::io;
use async_trait::async_trait;

pub use concurrent_listener::ConcurrentListener;
pub use failover_listener::FailoverListener;
pub use to_listener::ToListener;

pub(crate) use parsed_listener::ParsedListener;
pub(crate) use tcp_listener::TcpListener;
pub(crate) use unix_listener::UnixListener;

#[async_trait]
pub trait Listener<State>: Debug + Display + Send + Sync + 'static
where
    State: Send + Sync + 'static,
{
    async fn bind(&mut self, app: Server<State>) -> io::Result<()>;

    async fn accept(&mut self) -> io::Result<()>;

    fn info(&self) -> Vec<ListenInfo>;
}

#[async_trait]
impl<L, State> Listener<State> for Box<L>
where
    L: Listener<State>,
    State: Send + Sync + 'static,
{
    async fn bind(&mut self, app: Server<State>) -> io::Result<()> {
        self.as_mut().bind(app).await
    }

    async fn accept(&mut self) -> io::Result<()> {
        self.as_mut().accept().await
    }

    fn info(&self) -> Vec<ListenInfo> {
        self.as_ref().info()
    }
}

/// crate-internal shared logic used by tcp and unix listeners to
/// determine if an io::Error needs a backoff delay. Transient error
/// types do not require a delay.
pub(crate) fn is_transient_error(e: &io::Error) -> bool {
    use io::ErrorKind::*;

    matches!(
        e.kind(),
        ConnectionRefused | ConnectionAborted | ConnectionReset
    )
}

#[derive(Debug, Clone)]
pub struct ListenInfo {
    conn_string: String,
    transport: String,
    tls: bool,
}

impl ListenInfo {
    pub fn new(conn_string: String, transport: String, tls: bool) -> Self {
        Self {
            conn_string,
            transport,
            tls,
        }
    }

    pub fn connection(&self) -> &str {
        self.conn_string.as_str()
    }

    pub fn transport(&self) -> &str {
        self.transport.as_str()
    }

    pub fn is_encrypted(&self) -> bool {
        self.tls
    }
}

impl Display for ListenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.conn_string)
    }
}

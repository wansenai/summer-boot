#[cfg(unix)]
use super::UnixListener;
use super::{ListenInfo, Listener, TcpListener};
use crate::Server;

use async_std::io;
use std::fmt::{self, Debug, Display, Formatter};

pub enum ParsedListener<State> {
    Unix(UnixListener<State>),
    Tcp(TcpListener<State>),
}

impl<State> Debug for ParsedListener<State> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParsedListener::Unix(unix) => Debug::fmt(unix, f),
            ParsedListener::Tcp(tcp) => Debug::fmt(tcp, f),
        }
    }
}

impl<State> Display for ParsedListener<State> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unix(u) => write!(f, "{}", u),
            Self::Tcp(t) => write!(f, "{}", t),
        }
    }
}

#[async_trait::async_trait]
impl<State> Listener<State> for ParsedListener<State>
where
    State: Clone + Send + Sync + 'static,
{
    async fn bind(&mut self, server: Server<State>) -> io::Result<()> {
        match self {
            Self::Unix(u) => u.bind(server).await,
            Self::Tcp(t) => t.bind(server).await,
        }
    }

    async fn accept(&mut self) -> io::Result<()> {
        match self {
            Self::Unix(u) => u.accept().await,
            Self::Tcp(t) => t.accept().await,
        }
    }

    fn info(&self) -> Vec<ListenInfo> {
        match self {
            ParsedListener::Unix(unix) => unix.info(),
            ParsedListener::Tcp(tcp) => tcp.info(),
        }
    }
}

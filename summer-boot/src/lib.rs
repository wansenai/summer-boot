pub mod web2;
pub mod common;
pub mod log;

use async_std::io;
use serde_json::ser::State;
use web2::tcp::ToListener;
use web2::{
    utils,
    aop,
    gateway,
    context,
    tcp,
    http1,
};

pub use utils::middleware::{Middleware, Next};
pub use utils::request::Request;
pub use utils::response::Response;
pub use utils::response_builder::ResponseBuilder;
pub use utils::util;
pub use http1::http;

pub use http_types::{self, Body, Error, Status, StatusCode};
pub use aop::endpoint::Endpoint;
pub use gateway::route::Route;

use web2::server::server::Server;

#[must_use]
pub fn new() -> Server<()> {
    Server::new()
}

/// 自动扫描 日志开启 读取yml
pub async fn run() -> io::Result<()>
{
    Server::run().await
}

pub fn with_state<State>(state: State) -> Server<State>
where
    State: Clone + Copy + Send + Sync + 'static,
{
    Server::with_state(state)
}

/// 结果类型处理
pub type Result<T = Response> = std::result::Result<T, Error>;

pub mod rt;

/// 建立过程宏与summer boot的关联
macro_rules! macro_reexport {
    ($name:ident) => {
        #[cfg(feature = "macros")]
        #[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
        pub use summer_boot_macro::$name;
    };
}

macro_reexport!(auto_scan);
macro_reexport!(main);
macro_reexport!(post);
macro_reexport!(get);
macro_reexport!(delete);
macro_reexport!(put);
macro_reexport!(head);
macro_reexport!(options);
macro_reexport!(connect);
macro_reexport!(patch);
macro_reexport!(trace);

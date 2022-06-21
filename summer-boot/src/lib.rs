pub mod web2;
pub mod common;
pub mod log;

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

pub fn with_state<State>(state: State) -> Server<State>
where
    State: Clone + Send + Sync + 'static,
{
    Server::with_state(state)
}

/// 结果类型处理
pub type Result<T = Response> = std::result::Result<T, Error>;

pub mod rt;

/// 建立过程宏与summer boot的关联
macro_rules! codegen_reexport {
    ($name:ident) => {
        #[cfg(feature = "macros")]
        #[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
        pub use summer_boot_codegen::$name;
    };
}

codegen_reexport!(main);

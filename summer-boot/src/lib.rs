pub mod web2;
pub mod common;
pub mod log;

use web2::{
    utils,
    aop,
    gateway,
    context,
    tcp,
};

pub use utils::middleware::{Middleware, Next};
pub use utils::request::Request;
pub use utils::response::Response;
pub use utils::response_builder::ResponseBuilder;

pub use http_types::{self as http, Body, Error, Status, StatusCode};
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

/// A specialized Result type for Tide.
pub type Result<T = Response> = std::result::Result<T, Error>;

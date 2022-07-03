pub mod aop;
pub mod server;
///
/// 后续考虑把http1和http2 封装成rust的future
///
pub mod proto;
pub mod body;

#[macro_use]
pub mod common;

#[cfg(all(test, feature = "nightly"))]
extern crate test;

pub mod error;
pub mod ext;
pub mod headers;
pub mod mock;
pub mod rt;
pub mod upgrade;

#[macro_use]
pub mod macros;

cfg_proto! {
    mod headers;
    mod proto;
}

cfg_feature! {
    #![feature = "client"]

    pub mod client;
    #[cfg(any(feature = "http1", feature = "http2"))]
    #[doc(no_inline)]
    pub use crate::client::Client;
}

cfg_feature! {
    #![feature = "server"]

    pub mod server;
    #[doc(no_inline)]
    pub use crate::server::Server;
}
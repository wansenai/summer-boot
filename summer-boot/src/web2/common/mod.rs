macro_rules! ready {
    ($e:expr) => {
        match $e {
            std::task::Poll::Ready(v) => v,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}

pub(crate) mod buf;
#[cfg(all(feature = "server", any(feature = "http1", feature = "http2")))]
pub(crate) mod date;
#[cfg(all(feature = "server", any(feature = "http1", feature = "http2")))]
pub(crate) mod drain;
#[cfg(any(feature = "http1", feature = "http2", feature = "server"))]
pub(crate) mod exec;
pub(crate) mod io;
#[cfg(all(feature = "client", any(feature = "http1", feature = "http2")))]
mod lazy;
mod never;
pub(crate) mod exec;
#[cfg(any(
    feature = "stream",
    all(feature = "client", any(feature = "http1", feature = "http2"))
))]
pub(crate) mod sync_wrapper;
pub(crate) mod task;
pub(crate) mod watch;

#[cfg(all(feature = "client", any(feature = "http1", feature = "http2")))]
pub(crate) use self::lazy::{lazy, Started as Lazy};
#[cfg(any(feature = "http1", feature = "http2", feature = "runtime"))]
pub(crate) use self::never::Never;
pub(crate) use self::task::Poll;

macro_rules! cfg_feature {
    (
        #![$meta:meta]
        $($item:item)*
    ) => {
        $(
            #[cfg($meta)]
            #[cfg_attr(docsrs, doc(cfg($meta)))]
            $item
        )*
    }
}

macro_rules! cfg_proto {
    ($($item:item)*) => {
        cfg_feature! {
            #![all(
                any(feature = "http1", feature = "http2"),
                any(feature = "client", feature = "server"),
            )]
            $($item)*
        }
    }
}

cfg_proto! {
    macro_rules! cfg_client {
        ($($item:item)*) => {
            cfg_feature! {
                #![feature = "client"]
                $($item)*
            }
        }
    }

    macro_rules! cfg_server {
        ($($item:item)*) => {
            cfg_feature! {
                #![feature = "server"]
                $($item)*
            }
        }
    }
}

// `Future` 分组
cfg_proto! {
    pub(crate) use std::marker::Unpin;
}
pub(crate) use std::{future::Future, pin::Pin};
//! Pieces pertaining to the HTTP message protocol.
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

cfg_feature! {
    #![feature = "http1"]

    pub(crate) mod http1;

    pub(crate) use self::http1::Conn;

    #[cfg(feature = "client")]
    pub(crate) use self::http1::dispatch;
    #[cfg(feature = "server")]
    pub(crate) use self::http1::ServerTransaction;
}

#[cfg(feature = "http2")]
pub(crate) mod http2;

/// An Incoming Message head. Includes request/status line, and headers.
#[derive(Debug, Default)]
pub(crate) struct MessageHead<S> {
    /// HTTP version of the message.
    pub(crate) version: http::Version,
    /// Subject (request line or status line) of Incoming message.
    pub(crate) subject: S,
    /// Headers of the Incoming message.
    pub(crate) headers: http::HeaderMap,
    /// Extensions.
    extensions: http::Extensions,
}

/// An incoming request message.
#[cfg(feature = "http1")]
pub(crate) type RequestHead = MessageHead<RequestLine>;

#[derive(Debug, Default, PartialEq)]
#[cfg(feature = "http1")]
pub(crate) struct RequestLine(pub(crate) http::Method, pub(crate) http::Uri);

/// An incoming response message.
#[cfg(all(feature = "http1", feature = "client"))]
pub(crate) type ResponseHead = MessageHead<http::StatusCode>;

#[derive(Debug)]
#[cfg(feature = "http1")]
pub(crate) enum BodyLength {
    /// Content-Length
    Known(u64),
    /// Transfer-Encoding: chunked (if h1)
    Unknown,
}

/// Status of when a Disaptcher future completes.
pub(crate) enum Dispatched {
    /// Dispatcher completely shutdown connection.
    Shutdown,
    /// Dispatcher has pending upgrade, and so did not shutdown.
    #[cfg(feature = "http1")]
    Upgrade(crate::upgrade::Pending),
}

impl MessageHead<http::StatusCode> {
    fn into_response<B>(self, body: B) -> http::Response<B> {
        let mut res = http::Response::new(body);
        *res.status_mut() = self.subject;
        *res.headers_mut() = self.headers;
        *res.version_mut() = self.version;
        *res.extensions_mut() = self.extensions;
        res
    }
}
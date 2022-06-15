//! `Accept` trait 和支持的类型。
//! 
//! 这个模块包含:
//!
//! - 用于异步接受传入数据的 [`Accept`](Accept) feture。
//!   链接.
//! - 像 `poll_fn` 这样的程序可以创建自定义的 `Accept`.

#[cfg(feature = "stream")]
use futures_core::Stream;
#[cfg(feature = "stream")]
use pin_project_lite::pin_project;

use crate::common::{
    task::{self, Poll},
    Pin,
};

/// 异步接受传入连接。
pub trait Accept {
    /// 可以接受的连接类型。
    type Conn;
    /// 接受连接时可能发生的错误类型。
    type Error;

    /// 轮询接受下一个连接。
    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>>;
}

/// 使用轮询函数创建一个 `Accept` 。
/// # Example
///
pub fn poll_fn<F, IO, E>(func: F) -> impl Accept<Conn = IO, Error = E>
where
    F: FnMut(&mut task::Context<'_>) -> Poll<Option<Result<IO, E>>>,
{
    struct PollFn<F>(F);

    // 闭包 `F` 是不固定的
    impl<F> Unpin for PollFn<F> {}

    impl<F, IO, E> Accept for PollFn<F>
    where
        F: FnMut(&mut task::Context<'_>) -> Poll<Option<Result<IO, E>>>,
    {
        type Conn = IO;
        type Error = E;
        fn poll_accept(
            self: Pin<&mut Self>,
            cx: &mut task::Context<'_>,
        ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
            (self.get_mut().0)(cx)
        }
    }

    PollFn(func)
}

/// Adapt a `Stream` of incoming connections into an `Accept`.
/// 将传入连接的 `Stream` 改为 `Accept`。
///
/// # Optional
///
/// 此功能需要在
/// `Cargo.toml` feature进行配置。
#[cfg(feature = "stream")]
pub fn from_stream<S, IO, E>(stream: S) -> impl Accept<Conn = IO, Error = E>
where
    S: Stream<Item = Result<IO, E>>,
{
    pin_project! {
        struct FromStream<S> {
            #[pin]
            stream: S,
        }
    }

    impl<S, IO, E> Accept for FromStream<S>
    where
        S: Stream<Item = Result<IO, E>>,
    {
        type Conn = IO;
        type Error = E;
        fn poll_accept(
            self: Pin<&mut Self>,
            cx: &mut task::Context<'_>,
        ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
            self.project().stream.poll_next(cx)
        }
    }

    FromStream { stream }
}
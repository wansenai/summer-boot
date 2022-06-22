#[cfg(feature = "http1")]
use super::Never;
pub(crate) use std::task::{Context, Poll};

///
/// 重新安装feature
/// 这里用的是标准库Poll
///
#[cfg(feature = "http1")]
pub(crate) fn yield_now(cx: &mut Context<'_>) -> Poll<Never> {
    cx.waker().wake_by_ref();
    Poll::Pending
}

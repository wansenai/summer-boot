//!
//! 这里在service module使用
//! 可以考虑添加消除警告的属性宏
//!
pub(crate) mod task;
pub(crate) use self::task::Poll;
pub(crate) use std::pin::Pin;

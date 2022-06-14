pub(crate) mod task;
pub(crate) use std::{future::Future, pin::Pin};
pub(crate) use self::task::Poll;
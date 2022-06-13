//! 提供了summer boot的运行时环境
//! 当前提供环境主要是 tokio 下的 Runtime
//!

use tokio::runtime::Runtime;

/// 运行时简单代理对象
#[derive(Debug)]
pub struct SummerRuntime;

impl SummerRuntime {

    /// 新建 tokio runtime 运行时对象
    pub fn new() -> Runtime {
        tokio::runtime::Runtime::new().unwrap()
    }
}
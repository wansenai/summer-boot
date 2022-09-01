use super::Listener;
use async_std::io;

/// ToListener 可以转换为
/// [`Listener`](crate::listener::Listener)，实现的任何类型。
/// 现实可以看to_listener_impls
///
pub trait ToListener<State: Clone + Send + Sync + 'static> {
    /// 转换具体哪一种类型的Listener
    type Listener: Listener<State>;

    /// 将self进行转换为
    /// [`Listener`](crate::listener::Listener)。
    /// 除非self是已绑定/连接到io，转换为侦听器不启动连接。
    /// 错误返回表示转换为侦听器失败，而不是绑定尝试失败。
    fn to_listener(self) -> io::Result<Self::Listener>;
}

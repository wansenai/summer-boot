use crate::web2::tcp::{ListenInfo, Listener, ToListener};
use crate::Server;

use std::fmt::{self, Debug, Display, Formatter};

use async_std::io;
use futures_util::stream::{futures_unordered::FuturesUnordered, StreamExt};

#[derive(Default)]
pub struct ConcurrentListener<State> {
    listeners: Vec<Box<dyn Listener<State>>>,
}

impl<State: Clone + Send + Sync + 'static> ConcurrentListener<State> {
    pub fn new() -> Self {
        Self { listeners: vec![] }
    }

    pub fn add<L>(&mut self, listener: L) -> io::Result<()>
    where
        L: ToListener<State>,
    {
        self.listeners.push(Box::new(listener.to_listener()?));
        Ok(())
    }

    pub fn with_listener<L>(mut self, listener: L) -> Self
    where
        L: ToListener<State>,
    {
        self.add(listener).expect("无法添加侦听器");
        self
    }
}

#[async_trait::async_trait]
impl<State> Listener<State> for ConcurrentListener<State>
where
    State: Clone + Send + Sync + 'static,
{
    async fn bind(&mut self, app: Server<State>) -> io::Result<()> {
        for listener in self.listeners.iter_mut() {
            listener.bind(app.clone()).await?;
        }
        Ok(())
    }

    async fn accept(&mut self) -> io::Result<()> {
        let mut futures_unordered = FuturesUnordered::new();

        for listener in self.listeners.iter_mut() {
            futures_unordered.push(listener.accept());
        }

        while let Some(result) = futures_unordered.next().await {
            result?;
        }
        Ok(())
    }

    fn info(&self) -> Vec<ListenInfo> {
        self.listeners
            .iter()
            .flat_map(|listener| listener.info().into_iter())
            .collect()
    }
}

impl<State> Debug for ConcurrentListener<State> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.listeners)
    }
}

impl<State> Display for ConcurrentListener<State> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string = self
            .listeners
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(f, "{}", string)
    }
}

use crate::log;
use crate::{Body, Endpoint, Request, Response, Result, StatusCode};
use std::io;
use std::path::Path;

use async_std::path::PathBuf as AsyncPathBuf;
use async_trait::async_trait;

pub(crate) struct ServeFile {
    path: AsyncPathBuf,
}

impl ServeFile {
    /// 创建一个 `ServeFile` 新的实例。
    pub(crate) fn init(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = path.as_ref().to_owned().canonicalize()?;
        Ok(Self {
            path: AsyncPathBuf::from(file),
        })
    }
}

#[async_trait]
impl<State: Clone + Send + Sync + 'static> Endpoint<State> for ServeFile {
    async fn call(&self, _: Request<State>) -> Result {
        match Body::from_file(&self.path).await {
            Ok(body) => Ok(Response::builder(StatusCode::Ok).body(body).build()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                log::warn!("文件未找到: {:?}", &self.path);
                Ok(Response::new(StatusCode::NotFound))
            }
            Err(e) => Err(e.into()),
        }
    }
}
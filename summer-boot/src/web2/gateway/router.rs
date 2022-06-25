use crate::server;
use crate::{Request, Response, StatusCode};

use routefinder::{Captures, Router as MethodRouter};
use std::collections::HashMap;

use server::endpoint::DynEndpoint;

/// `Server` 使用的路由
///
/// 底层, 每个HTTP方法都有一个单独的状态；索引
/// 通过该方法，可以提高效率
#[allow(missing_debug_implementations)]
pub(crate) struct Router<State> {
    method_map: HashMap<http_types::Method, MethodRouter<Box<DynEndpoint<State>>>>,
    all_method_router: MethodRouter<Box<DynEndpoint<State>>>,
}

impl<State> std::fmt::Debug for Router<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Router")
            .field("method_map", &self.method_map)
            .field("all_method_router", &self.all_method_router)
            .finish()
    }
}

/// 路由URL的结果
pub(crate) struct Selection<'a, State> {
    pub(crate) endpoint: &'a DynEndpoint<State>,
    pub(crate) params: Captures<'static, 'static>,
}

impl<State: Clone + Send + Sync + 'static> Router<State> {
    pub(crate) fn new() -> Self {
        Router {
            method_map: HashMap::default(),
            all_method_router: MethodRouter::new(),
        }
    }

    pub(crate) fn add(
        &mut self,
        path: &str,
        method: http_types::Method,
        ep: Box<DynEndpoint<State>>,
    ) {
        self.method_map
            .entry(method)
            .or_insert_with(MethodRouter::new)
            .add(path, ep)
            .unwrap()
    }

    pub(crate) fn add_all(&mut self, path: &str, ep: Box<DynEndpoint<State>>) {
        self.all_method_router.add(path, ep).unwrap()
    }

    pub(crate) fn route(&self, path: &str, method: http_types::Method) -> Selection<'_, State> {
        if let Some(m) = self
            .method_map
            .get(&method)
            .and_then(|r| r.best_match(path))
        {
            Selection {
                endpoint: m.handler(),
                params: m.captures().into_owned(),
            }
        } else if let Some(m) = self.all_method_router.best_match(path) {
            Selection {
                endpoint: m.handler(),
                params: m.captures().into_owned(),
            }
        } else if method == http_types::Method::Head {
            // 如果是HTTP头请求，则检查endpoints映射中是否有回调
            // 如果没有，则返回到HTTP GET的逻辑，否则照常进行

            self.route(path, http_types::Method::Get)
        } else if self
            .method_map
            .iter()
            .filter(|(k, _)| **k != method)
            .any(|(_, r)| r.best_match(path).is_some())
        {
            // 如果此 `path` 可以由使用其他HTTP方法注册的回调处理
            // 应返回405 Method Not Allowed
            Selection {
                endpoint: &method_not_allowed,
                params: Captures::default(),
            }
        } else {
            Selection {
                endpoint: &not_found_endpoint,
                params: Captures::default(),
            }
        }
    }
}

async fn not_found_endpoint<State: Clone + Send + Sync + 'static>(
    _req: Request<State>,
) -> crate::Result {
    Ok(Response::new(StatusCode::NotFound))
}

async fn method_not_allowed<State: Clone + Send + Sync + 'static>(
    _req: Request<State>,
) -> crate::Result {
    Ok(Response::new(StatusCode::MethodNotAllowed))
}

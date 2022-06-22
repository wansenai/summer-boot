//! HTTP1 connections on the server.

use std::str::FromStr;
use std::task::{Context, Poll};
use std::{fmt, marker::PhantomData, pin::Pin, time::Duration};

use async_std::future::{timeout, Future, TimeoutError};
use async_std::io::{self, BufRead, BufReader, Read, Take, Write};
use async_std::{prelude::*, task};

use http_types::content::ContentLength;
use http_types::headers::{CONNECTION, EXPECT, TRANSFER_ENCODING, UPGRADE};
use http_types::upgrade::Connection;
use http_types::{ensure, ensure_eq, format_err};
use http_types::{Body, Method, Request, Response, StatusCode, Url};

use async_channel::Sender;
use async_dup::{Arc, Mutex};

use super::decode::ChunkedDecoder;
use super::encode::Encoder;

const MAX_HEADERS: usize = 128;
const MAX_HEAD_LENGTH: usize = 8 * 1024;

const LF: u8 = b'\n';

/// 当请求为HTTP 1.1时，从httparse返回的数字
const HTTP_1_1_VERSION: u8 = 1;

const CONTINUE_HEADER_VALUE: &str = "100-continue";
const CONTINUE_RESPONSE: &[u8] = b"HTTP/1.1 100 Continue\r\n\r\n";

// http1 connection 配置服务器
#[derive(Debug, Clone)]
pub struct ServerOptions {
    /// 处理headers超时。默认值为60秒
    headers_timeout: Option<Duration>,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            headers_timeout: Some(Duration::from_secs(60)),
        }
    }
}

/// 接受新的传入HTTP/1.1连接
/// 默认情况支持KeepAlive请求。
pub async fn accept<RW, F, Fut>(io: RW, endpoint: F) -> http_types::Result<()>
where
    RW: Read + Write + Clone + Send + Sync + Unpin + 'static,
    F: Fn(Request) -> Fut,
    Fut: Future<Output = http_types::Result<Response>>,
{
    Server::new(io, endpoint).accept().await
}

/// 接受新的传入HTTP/1.1连接
/// 默认情况支持KeepAlive请求。
pub async fn accept_with_opts<RW, F, Fut>(
    io: RW,
    endpoint: F,
    opts: ServerOptions,
) -> http_types::Result<()>
where
    RW: Read + Write + Clone + Send + Sync + Unpin + 'static,
    F: Fn(Request) -> Fut,
    Fut: Future<Output = http_types::Result<Response>>,
{
    Server::new(io, endpoint).with_opts(opts).accept().await
}

/// struct server
#[derive(Debug)]
pub struct Server<RW, F, Fut> {
    io: RW,
    endpoint: F,
    opts: ServerOptions,
    _phantom: PhantomData<Fut>,
}

/// 服务器是否应接受后续请求的枚举
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConnectionStatus {
    /// 服务器不接受其他请求
    Close,

    /// 服务器可能会接受另一个请求
    KeepAlive,
}

impl<RW, F, Fut> Server<RW, F, Fut>
where
    RW: Read + Write + Clone + Send + Sync + Unpin + 'static,
    F: Fn(Request) -> Fut,
    Fut: Future<Output = http_types::Result<Response>>,
{
    ///构建一个新服务器
    pub fn new(io: RW, endpoint: F) -> Self {
        Self {
            io,
            endpoint,
            opts: Default::default(),
            _phantom: PhantomData,
        }
    }

    /// with opts
    pub fn with_opts(mut self, opts: ServerOptions) -> Self {
        self.opts = opts;
        self
    }

    /// accept in a loop
    pub async fn accept(&mut self) -> http_types::Result<()> {
        while ConnectionStatus::KeepAlive == self.accept_one().await? {}
        Ok(())
    }

    /// accept one request
    pub async fn accept_one(&mut self) -> http_types::Result<ConnectionStatus>
    where
        RW: Read + Write + Clone + Send + Sync + Unpin + 'static,
        F: Fn(Request) -> Fut,
        Fut: Future<Output = http_types::Result<Response>>,
    {
        // 对新请求进行解码，如果解码时间超过超时持续时间，则超时。
        let fut = decode(self.io.clone());

        let (req, mut body) = if let Some(timeout_duration) = self.opts.headers_timeout {
            match timeout(timeout_duration, fut).await {
                Ok(Ok(Some(r))) => r,
                Ok(Ok(None)) | Err(TimeoutError { .. }) => return Ok(ConnectionStatus::Close), /* EOF或超时 */
                Ok(Err(e)) => return Err(e),
            }
        } else {
            match fut.await? {
                Some(r) => r,
                None => return Ok(ConnectionStatus::Close), /* EOF */
            }
        };

        let has_upgrade_header = req.header(UPGRADE).is_some();
        let connection_header_as_str = req
            .header(CONNECTION)
            .map(|connection| connection.as_str())
            .unwrap_or("");

        let connection_header_is_upgrade = connection_header_as_str
            .split(',')
            .any(|s| s.trim().eq_ignore_ascii_case("upgrade"));
        let mut close_connection = connection_header_as_str.eq_ignore_ascii_case("close");

        let upgrade_requested = has_upgrade_header && connection_header_is_upgrade;

        let method = req.method();

        // 将请求传递给endpoint并对响应进行编码
        let mut res = (self.endpoint)(req).await?;

        close_connection |= res
            .header(CONNECTION)
            .map(|c| c.as_str().eq_ignore_ascii_case("close"))
            .unwrap_or(false);

        let upgrade_provided = res.status() == StatusCode::SwitchingProtocols && res.has_upgrade();

        let upgrade_sender = if upgrade_requested && upgrade_provided {
            Some(res.send_upgrade())
        } else {
            None
        };

        let mut encoder = Encoder::new(res, method);

        let bytes_written = io::copy(&mut encoder, &mut self.io).await?;
        log::trace!("wrote {} response bytes", bytes_written);

        let body_bytes_discarded = io::copy(&mut body, &mut io::sink()).await?;
        log::trace!(
            "discarded {} unread request body bytes",
            body_bytes_discarded
        );

        if let Some(upgrade_sender) = upgrade_sender {
            upgrade_sender.send(Connection::new(self.io.clone())).await;
            Ok(ConnectionStatus::Close)
        } else if close_connection {
            Ok(ConnectionStatus::Close)
        } else {
            Ok(ConnectionStatus::KeepAlive)
        }
    }
}

/// body_reader
pub enum BodyReader<IO: Read + Unpin> {
    Chunked(Arc<Mutex<ChunkedDecoder<BufReader<IO>>>>),
    Fixed(Arc<Mutex<Take<BufReader<IO>>>>),
    None,
}

impl<IO: Read + Unpin> fmt::Debug for BodyReader<IO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BodyReader::Chunked(_) => f.write_str("BodyReader::Chunked"),
            BodyReader::Fixed(_) => f.write_str("BodyReader::Fixed"),
            BodyReader::None => f.write_str("BodyReader::None"),
        }
    }
}

impl<IO: Read + Unpin> Read for BodyReader<IO> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match &*self {
            BodyReader::Chunked(r) => Pin::new(&mut *r.lock()).poll_read(cx, buf),
            BodyReader::Fixed(r) => Pin::new(&mut *r.lock()).poll_read(cx, buf),
            BodyReader::None => Poll::Ready(Ok(0)),
        }
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [io::IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        for b in bufs {
            if !b.is_empty() {
                return self.poll_read(cx, b);
            }
        }

        self.poll_read(cx, &mut [])
    }
}

/// read_notifier
#[pin_project::pin_project]
pub struct ReadNotifier<B> {
    #[pin]
    reader: B,
    sender: Sender<()>,
    has_been_read: bool,
}

impl<B> fmt::Debug for ReadNotifier<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadNotifier")
            .field("read", &self.has_been_read)
            .finish()
    }
}

impl<B: Read> ReadNotifier<B> {
    pub(crate) fn new(reader: B, sender: Sender<()>) -> Self {
        Self {
            reader,
            sender,
            has_been_read: false,
        }
    }
}

impl<B: BufRead> BufRead for ReadNotifier<B> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        self.project().reader.poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().reader.consume(amt)
    }
}

impl<B: Read> Read for ReadNotifier<B> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();

        if !*this.has_been_read {
            if let Ok(()) = this.sender.try_send(()) {
                *this.has_been_read = true;
            };
        }

        this.reader.poll_read(cx, buf)
    }
}

/// 解码服务器上的HTTP请求
pub async fn decode<IO>(mut io: IO) -> http_types::Result<Option<(Request, BodyReader<IO>)>>
where
    IO: Read + Write + Clone + Send + Sync + Unpin + 'static,
{
    let mut reader = BufReader::new(io.clone());
    let mut buf = Vec::new();
    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut httparse_req = httparse::Request::new(&mut headers);

    // 一直从流中读取字节，直到到达流快结束的时候
    loop {
        let bytes_read = reader.read_until(LF, &mut buf).await?;
        // 不再从流中生成更多字节
        if bytes_read == 0 {
            return Ok(None);
        }

        // 防止DDOS
        ensure!(
            buf.len() < MAX_HEAD_LENGTH,
            "Head byte length should be less than 8kb"
        );

        // 找到了流的结束分割符
        let idx = buf.len() - 1;
        if idx >= 3 && &buf[idx - 3..=idx] == b"\r\n\r\n" {
            break;
        }
    }

    // 将header buf转换为httparse实例，并进行验证
    let status = httparse_req.parse(&buf)?;

    ensure!(!status.is_partial(), "Malformed HTTP head");

    // 将httparse headers + body 转换为 `http_types::Request` 类型。
    let method = httparse_req.method;
    let method = method.ok_or_else(|| format_err!("No method found"))?;

    let version = httparse_req.version;
    let version = version.ok_or_else(|| format_err!("No version found"))?;

    ensure_eq!(
        version,
        HTTP_1_1_VERSION,
        "Unsupported HTTP version 1.{}",
        version
    );

    let url = url_from_httparse_req(&httparse_req)?;

    let mut req = Request::new(Method::from_str(method)?, url);

    req.set_version(Some(http_types::Version::Http1_1));

    for header in httparse_req.headers.iter() {
        req.append_header(header.name, std::str::from_utf8(header.value)?);
    }

    let content_length = ContentLength::from_headers(&req)?;
    let transfer_encoding = req.header(TRANSFER_ENCODING);

    // 如果内容长度和传输编码头都是，则返回400状态
    // 设置为防止请求攻击。
    //
    // https://tools.ietf.org/html/rfc7230#section-3.3.3
    http_types::ensure_status!(
        content_length.is_none() || transfer_encoding.is_none(),
        400,
        "Unexpected Content-Length header"
    );

    // 建立一个通道以等待读取body, 允许我们避免在以下情况下发送100-continue
    // 无需读取body即可响应，避免客户端上传body
    let (body_read_sender, body_read_receiver) = async_channel::bounded(1);

    if Some(CONTINUE_HEADER_VALUE) == req.header(EXPECT).map(|h| h.as_str()) {
        task::spawn(async move {
            // /如果客户端需要100 continue标头，则生成任务等待正文上的第一次读取尝试。
            if let Ok(()) = body_read_receiver.recv().await {
                io.write_all(CONTINUE_RESPONSE).await.ok();
            };
            // 由于发件方已移动到body中，因此此任务将 在客户端断开连接时完成，无论发送了100-continue
        });
    }

    // 检查传输编码
    if transfer_encoding
        .map(|te| te.as_str().eq_ignore_ascii_case("chunked"))
        .unwrap_or(false)
    {
        let trailer_sender = req.send_trailers();
        let reader = ChunkedDecoder::new(reader, trailer_sender);
        let reader = Arc::new(Mutex::new(reader));
        let reader_clone = reader.clone();
        let reader = ReadNotifier::new(reader, body_read_sender);
        let reader = BufReader::new(reader);
        req.set_body(Body::from_reader(reader, None));
        Ok(Some((req, BodyReader::Chunked(reader_clone))))
    } else if let Some(len) = content_length {
        let len = len.len();
        let reader = Arc::new(Mutex::new(reader.take(len)));
        req.set_body(Body::from_reader(
            BufReader::new(ReadNotifier::new(reader.clone(), body_read_sender)),
            Some(len as usize),
        ));
        Ok(Some((req, BodyReader::Fixed(reader))))
    } else {
        Ok(Some((req, BodyReader::None)))
    }
}

fn url_from_httparse_req(req: &httparse::Request<'_, '_>) -> http_types::Result<Url> {
    let path = req.path.ok_or_else(|| format_err!("No uri found"))?;

    let host = req
        .headers
        .iter()
        .find(|x| x.name.eq_ignore_ascii_case("host"))
        .ok_or_else(|| format_err!("Mandatory Host header missing"))?
        .value;

    let host = std::str::from_utf8(host)?;

    if path.starts_with("http://") || path.starts_with("https://") {
        Ok(Url::parse(path)?)
    } else if path.starts_with('/') {
        Ok(Url::parse(&format!("http://{}{}", host, path))?)
    } else if req.method.unwrap().eq_ignore_ascii_case("connect") {
        Ok(Url::parse(&format!("http://{}/", path))?)
    } else {
        Err(format_err!("unexpected uri format"))
    }
}

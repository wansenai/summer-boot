use std::io::Write;
use std::pin::Pin;
use std::time::SystemTime;

use crate::read_to_end;
use async_std::io::{self, Cursor, Read};
use async_std::task::{Context, Poll};
use futures_util::ready;
use http_types::headers::{CONTENT_LENGTH, DATE, TRANSFER_ENCODING};
use http_types::{Body, Method, Response};
use pin_project::pin_project;

use super::body_encoder::BodyEncoder;
use super::date::fmt_http_date;

#[derive(Debug)]
pub(crate) enum EncoderState {
    Start,
    Head(Cursor<Vec<u8>>),
    Body(BodyEncoder),
    End,
}

/// streaming HTTP 编码
#[derive(Debug)]
pub struct Encoder {
    response: Response,
    state: EncoderState,
    method: Method,
}

impl Read for Encoder {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            self.state = match self.state {
                EncoderState::Start => EncoderState::Head(self.compute_head()?),

                EncoderState::Head(ref mut cursor) => {
                    read_to_end!(Pin::new(cursor).poll_read(cx, buf));

                    if self.method == Method::Head {
                        EncoderState::End
                    } else {
                        EncoderState::Body(BodyEncoder::new(self.response.take_body()))
                    }
                }

                EncoderState::Body(ref mut encoder) => {
                    read_to_end!(Pin::new(encoder).poll_read(cx, buf));
                    EncoderState::End
                }

                EncoderState::End => return Poll::Ready(Ok(0)),
            }
        }
    }
}

impl Encoder {
    /// 创建编码的新实例。
    pub fn new(response: Response, method: Method) -> Self {
        Self {
            method,
            response,
            state: EncoderState::Start,
        }
    }

    fn finalize_headers(&mut self) {
        // 如果正文没有流传输，可以提前设置内容长度。否则需要分块发送所有
        if let Some(len) = self.response.len() {
            self.response.insert_header(CONTENT_LENGTH, len.to_string());
        } else {
            self.response.insert_header(TRANSFER_ENCODING, "chunked");
        }

        if self.response.header(DATE).is_none() {
            let date = fmt_http_date(SystemTime::now());
            self.response.insert_header(DATE, date);
        }
    }

    /// 第一次轮询时，将header编码到缓冲区。
    fn compute_head(&mut self) -> io::Result<Cursor<Vec<u8>>> {
        let mut head = Vec::with_capacity(128);
        let reason = self.response.status().canonical_reason();
        let status = self.response.status();
        write!(head, "HTTP/1.1 {} {}\r\n", status, reason)?;

        self.finalize_headers();
        let mut headers = self.response.iter().collect::<Vec<_>>();
        headers.sort_unstable_by_key(|(h, _)| h.as_str());
        for (header, values) in headers {
            for value in values.iter() {
                write!(head, "{}: {}\r\n", header, value)?;
            }
        }
        write!(head, "\r\n")?;
        Ok(Cursor::new(head))
    }
}

/// 用于分块编码的编码struct
#[derive(Debug)]
pub(crate) struct ChunkedEncoder<R> {
    reader: R,
    done: bool,
}

impl<R: Read + Unpin> ChunkedEncoder<R> {
    /// 创建一个新的实例
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader,
            done: false,
        }
    }
}

impl<R: Read + Unpin> Read for ChunkedEncoder<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if self.done {
            return Poll::Ready(Ok(0));
        }
        let reader = &mut self.reader;

        let max_bytes_to_read = max_bytes_to_read(buf.len());

        let bytes = ready!(Pin::new(reader).poll_read(cx, &mut buf[..max_bytes_to_read]))?;
        if bytes == 0 {
            self.done = true;
        }
        let start = format!("{:X}\r\n", bytes);
        let start_length = start.as_bytes().len();
        let total = bytes + start_length + 2;
        buf.copy_within(..bytes, start_length);
        buf[..start_length].copy_from_slice(start.as_bytes());
        buf[total - 2..total].copy_from_slice(b"\r\n");
        Poll::Ready(Ok(total))
    }
}

fn max_bytes_to_read(buf_len: usize) -> usize {
    if buf_len < 6 {
        // 最小读取大小为6表示正文中的内容。其他五个字节是 1\r\n\r\n
        //其中 _ 是实际内容
        panic!("buffers of length {} are too small for this implementation. if this is a problem for you, please open an issue", buf_len);
    }

    let bytes_remaining_after_two_cr_lns = (buf_len - 4) as f64;

    // the maximum number of bytes that the hex representation of remaining bytes might take
    let max_bytes_of_hex_framing = bytes_remaining_after_two_cr_lns.log2() / 4f64;

    (bytes_remaining_after_two_cr_lns - max_bytes_of_hex_framing.ceil()) as usize
}

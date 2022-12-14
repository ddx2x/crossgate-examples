// use async_trait::async_trait;
use bytes::Buf;
use crossgate_rs::net::{Connection, ConnectionError};
use std::io::{BufRead, Write};

#[derive(Debug, Clone)]
pub enum EchoFrame {
    Request(String),
    Response(String),
    Unknown(String),
    None,
}

impl crossgate_rs::net::Frame for EchoFrame {
    fn read(&self, buf: &mut std::io::Cursor<&[u8]>) -> Result<Self, crossgate_rs::net::FrameError>
    where
        Self: std::marker::Sized,
    {
        if !buf.has_remaining() {
            return Err(crossgate_rs::net::FrameError::Incomplete);
        }
        let mut line = String::new();
        if let Err(e) = buf.read_line(&mut line) {
            return Err(crossgate_rs::net::FrameError::ParseError(e.to_string()));
        }

        match line.trim() {
            "hello" => Ok(EchoFrame::Response("world\r\n".to_string())),
            "haha" => Ok(EchoFrame::Response("xixi\r\n".to_string())),
            "bye" | "quit" => Err(crossgate_rs::net::FrameError::Exit),
            _ => Err(crossgate_rs::net::FrameError::ParseError(
                line.trim().to_string(),
            )),
        }
    }

    fn write<W>(&self, w: &mut W) -> Result<(), crossgate_rs::net::FrameError>
    where
        W: Write,
    {
        match self {
            EchoFrame::Response(s) => w
                .write_all(s.as_bytes())
                .map_err(|e| crossgate_rs::net::FrameError::ParseError(e.to_string())),
            _ => w.write_all("unknown command\r\n".as_bytes()).map_err(|e| {
                crossgate_rs::net::FrameError::Other(crossgate_rs::net::NetError::InternalError(
                    e.to_string(),
                ))
            }),
        }
    }
}

#[derive(Clone)]
pub struct EchoFrameHandle {}

impl crossgate_rs::net::Handle for EchoFrameHandle {
    type HandleFuture<'a> = impl std::future::Future<Output = Result<(), ConnectionError>> + 'a
    where
        Self: 'a;

    fn handle<'r>(self, conn: &'r mut Connection) -> Self::HandleFuture<'r> {
        let block = async move {
            while let Some(frame) = conn.read_frame(&EchoFrame::Request("".to_string())).await? {
                let _ = conn.write_frame(frame).await?;
            }
            Ok(())
        };
        block
    }
}

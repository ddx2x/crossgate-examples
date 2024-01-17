use bytes::Buf;
use crossgate_rs::net::{Connection, ConnectionError};
use std::io::Read;

#[derive(Clone, Debug)]
pub enum JT808Frame {
    P0200([u8; 8]), // 64bit [0..64]
    None,
}

impl crossgate_rs::net::Frame for JT808Frame {
    fn read(&self, buf: &mut std::io::Cursor<&[u8]>) -> Result<Self, crossgate_rs::net::FrameError>
    where
        Self: std::marker::Sized,
    {
        if !buf.has_remaining() {
            return Err(crossgate_rs::net::FrameError::Incomplete);
        }

        match buf.get_u8() {
            b'~' => {
                let mut data = [0u8; 8];
                if let Ok(_) = buf.read_exact(&mut data) {
                    return Ok(JT808Frame::P0200(data));
                }
                return Ok(JT808Frame::None);
            }
            _ => Ok(JT808Frame::None),
        }
    }

    fn write<W>(&self, _w: &mut W) -> Result<(), crossgate_rs::net::FrameError>
    where
        W: std::io::Write,
    {
        todo!()
    }
}

#[derive(Clone)]
pub struct JT808Handle {}

impl crossgate_rs::net::Handle for JT808Handle {
    type HandleFuture<'a> = impl futures::Future<Output = Result<(), ConnectionError>> + 'a
    where
        Self: 'a;

    fn handle<'r>(self, conn: &'r mut Connection) -> Self::HandleFuture<'r> {
        let block = async move {
            while let Some(frame) = conn.read_frame::<JT808Frame>(&JT808Frame::None).await? {
                match frame {
                    JT808Frame::P0200(data) => {
                        println!("{:?}", data);
                    }
                    _ => {}
                }
            }
            Ok(())
        };

        block
    }
}

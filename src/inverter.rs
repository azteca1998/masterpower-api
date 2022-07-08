use crate::codec::Codec;
use crate::command::Command;
use crate::error::{Error, Result};
use bytes::{Buf, BytesMut};
use log::trace;
use tokio::io::{AsyncRead, AsyncWrite, ErrorKind};
use tokio::prelude::*;
use tokio_util::codec::{Decoder, Encoder};

// TODO: Find a better way to temporarily move out of the struct (within the same method).
pub struct Inverter<S> {
    stream: S,
    buffer_in: BytesMut,
}

impl<S> Inverter<S> {
    pub fn from_stream(stream: S) -> Self {
        Self {
            stream,
            buffer_in: BytesMut::new(),
        }
    }

    pub fn into_inner(self) -> S {
        // WARNING: From this point onwards, `buffer_in` is lost! Reading may not be synchronized.
        self.stream
    }
}

impl<S> Inverter<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    #[allow(unused)]
    pub async fn execute<C: Command>(&mut self, req: C::Request) -> Result<C::Response> {
        // TODO: Find a way to use the `Framed` facility.

        // Encode the message.
        let mut codec = Codec::<C>::new();

        let mut buf = BytesMut::new();
        codec.encode(req, &mut buf)?;

        trace!("Writing command to stream");
        self.stream.flush().await?;
        self.stream.write_all(buf.bytes()).await?;
        // trace!("Command written successfully");

        // trace!("Buffer contents before first read: {:?}", self.buffer_in);
        Ok(loop {
            self.buffer_in.reserve(1024);
            let len = self.stream.read_buf(&mut self.buffer_in).await?;
            if len == 0 {
                return Err(Error::Io(ErrorKind::UnexpectedEof.into()));
            }

            // trace!("Read {} bytes from stream", len);
            // trace!("Calling decode with stream: {:?}", self.buffer_in);

            if let Some(item) = codec.decode(&mut self.buffer_in)? {
                break item;
            }
        })
    }
}

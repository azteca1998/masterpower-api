use crate::command::{Command, Request, Response};
use crate::error::{Error, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crc_any::CRCu16;
use log::{debug, trace};
use std::marker::PhantomData;
use std::mem::size_of;
use tokio_util::codec::{Decoder, Encoder};

pub struct Codec<C> {
    phantom: PhantomData<C>,
}

impl<C> Codec<C> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData::default(),
        }
    }
}

impl<C> Decoder for Codec<C>
where
    C: Command,
{
    type Item = C::Response;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        let maybe_index = src
            .iter()
            .copied()
            .enumerate()
            .find(|(_, x)| *x == '\r' as u8)
            .map(|(i, _)| i);

        Ok(match maybe_index {
            Some(index) => {
                let mut item = src.split_to(index + 1);

                debug!("Decoding response {}.", C::COMMAND_NAME);
                trace!("Decoding response ({}): {:?}.", C::COMMAND_NAME, &item[..]);

                let suffix = item.split_off(item.len() - size_of::<u8>());
                if suffix != Bytes::from_static(b"\r") {
                    unreachable!()
                }

                let crc_sum = item.split_off(item.len() - size_of::<u16>()).get_u16();
                let mut computed_crc_sum = CRCu16::crc16xmodem();
                computed_crc_sum.digest(item.bytes());
                if crc_sum != computed_crc_sum.get_crc() {
                    return Err(Error::InvalidResponseCrcSum);
                }

                let prefix = item.split_to(size_of::<u8>());
                if prefix != Bytes::from_static(b"(") {
                    return Err(Error::InvalidResponsePrefix);
                }

                let item = C::Response::decode(&mut item)?;
                trace!("Decoded response ({}): {:?}", C::COMMAND_NAME, item);

                Some(item)
            }
            None => None,
        })
    }
}

impl<C> Encoder<C::Request> for Codec<C>
where
    C: Command,
{
    type Error = Error;

    fn encode(&mut self, item: C::Request, dst: &mut BytesMut) -> Result<()> {
        // Note: Used for logging.
        let start_len = dst.len();

        debug!("Encoding command {}.", C::COMMAND_NAME);
        trace!("Command payload ({}): {:?}.", C::COMMAND_NAME, item);

        // Put the command id.
        dst.put_slice(C::PROTOCOL_ID);

        // Put the request payload (if any).
        if let Some(request_payload) = item.encode()? {
            dst.put(request_payload);
        }

        // Compute the CRC sum.
        let mut crc_sum = CRCu16::crc16xmodem();
        crc_sum.digest(dst.bytes());
        let crc_sum = crc_sum.get_crc();

        // Put the CRC sum.
        // Put the carriage return.
        dst.put_u16(crc_sum);
        dst.put_u8('\r' as u8);

        trace!(
            "Encoded command ({}): {:?}",
            C::COMMAND_NAME,
            &dst[start_len..]
        );

        Ok(())
    }
}

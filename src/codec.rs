use crate::command::{Request, Response};
use crate::error::{Error, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crc_any::CRCu16;
use log::{debug, trace};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::size_of;
use tokio_util::codec::{Decoder, Encoder};

pub struct Codec<Q, R> {
    phantom_q: PhantomData<Q>,
    phantom_r: PhantomData<R>,
}

impl<Q, R> Decoder for Codec<Q, R>
where
    R: Debug + Response,
{
    type Item = R;
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

                let prefix = item.split_to(size_of::<u8>());
                if prefix != Bytes::from_static(b"(") {
                    return Err(Error::InvalidResponsePrefix);
                }

                let suffix = item.split_off(size_of::<u8>());
                if suffix != Bytes::from_static(b"\r") {
                    unreachable!()
                }

                let crc_sum = item.split_off(size_of::<u16>()).get_u16();
                let mut computed_crc_sum = CRCu16::crc16xmodem();
                computed_crc_sum.digest(item.bytes());
                if crc_sum != computed_crc_sum.get_crc() {
                    return Err(Error::InvalidResponseCrcSum);
                }

                Some(R::decode(&mut item)?)
            }
            None => None,
        })
    }
}

impl<Q, R> Encoder<Q> for Codec<Q, R>
where
    Q: Debug + Request,
{
    type Error = Error;

    fn encode(&mut self, item: Q, dst: &mut BytesMut) -> Result<()> {
        debug!("Encoding command {}.", Q::COMMAND_NAME);
        trace!("Command payload: {:?}.", item);

        // Put the command id.
        dst.put_slice(Q::PROTOCOL_ID);

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

        Ok(())
    }
}

use crate::command::{Command, Request, Response};
use crate::error::{Error, Result};
use bytes::{Buf, BufMut, BytesMut};
use crc_any::CRCu16;
use log::{debug, trace};
use std::marker::PhantomData;
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

    fn compute_crc(data: &[u8]) -> u16 {
        let mut computed_crc_sum = CRCu16::crc16xmodem();
        computed_crc_sum.digest(data);

        computed_crc_sum.get_crc()
    }
}

impl<C> Default for Codec<C> {
    fn default() -> Self {
        Self::new()
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
            .find(|(_, x)| *x == b'\r')
            .map(|(i, _)| i);

        Ok(match maybe_index {
            Some(index) => {
                let mut item = &src[..index];
                let mut recover_length = 0;
                if *item.first().unwrap() != b'(' {
                    trace!(
                        "Invalid response format ({}): {:?}",
                        C::COMMAND_NAME,
                        &item[..]
                    );

                    // Try to recover by removing everything until the first (
                    let index = item.iter().position(|&r| r == b'(');

                    if index.is_none() {
                        src.advance(src.len());
                        return Err(Error::InvalidResponseFormat);
                    }

                    trace!("Attempting to recover from invalid response");
                    let split = item.split_at(index.unwrap());
                    recover_length = split.0.len();
                    item = split.1;
                }

                debug!("Decoding response {}.", C::COMMAND_NAME);
                trace!("Decoding response ({}): {:?}.", C::COMMAND_NAME, &item[..]);

                for step in 0..3 {
                    // Check the CRC for the current detected payload.
                    let crc_sum = Self::compute_crc(&item[..item.len() - 2]);
                    if crc_sum == (&item[item.len() - 2..]).get_u16() {
                        break;
                    }

                    // Check src length.
                    // Ensure the new command response ends in \r.
                    if index + step + 1 == src.len()
                        || !match step {
                            0 => {
                                let check = src[index + 1] == b'\r';
                                if !check
                                    && (index + step + 2 < src.len() && src[index + 2] == b'\r')
                                {
                                    continue;
                                }

                                check
                            }
                            1 => src[index + 2] == b'\r',
                            _ => unimplemented!(),
                        }
                    {
                        src.advance(src.len());
                        return Err(Error::InvalidResponseCrcSum);
                    }

                    item = &src[..index + step + 1];
                }

                // TODO: Do this without copying memory.
                let decoded_item =
                    C::Response::decode(&mut BytesMut::from(&item[1..item.len() - 2]))?;
                trace!("Decoded response ({}): {:?}", C::COMMAND_NAME, decoded_item);

                let item_len = item.len();
                src.advance(item_len + recover_length + 1);

                Some(decoded_item)
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
        dst.put_u8(b'\r');

        trace!(
            "Encoded command ({}): {:?}",
            C::COMMAND_NAME,
            &dst[start_len..]
        );

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::codec::Codec;
    use crate::commands::qid::{QIDResponse, QID};
    use crate::error::Result;
    use bytes::BytesMut;
    use crc_any::CRCu16;
    use tokio_util::codec::Decoder;

    #[test]
    fn test_decode_invalid_format() -> Result<()> {
        let mut codec = Codec::<QID>::new();

        let mut res = String::from("123 123").into_bytes();
        let mut crc_sum = CRCu16::crc16xmodem();
        crc_sum.digest(res.as_slice());
        res.extend_from_slice(crc_sum.get_crc().to_be_bytes().as_ref());
        res.push(b'\r');

        let mut buf = BytesMut::from(res.as_slice());
        let item = codec.decode(&mut buf);

        assert!(item.is_err());
        // assert_eq!(item, Err(Error::InvalidResponseFormat));

        Ok(())
    }

    #[test]
    fn test_recover_invalid_format() -> Result<()> {
        let mut codec = Codec::<QID>::new();
        // let mut res = String::from("\x00\x00\x00\x00\x00\x00(001.0 00.0 229.0 50.0 0092 0092 003 420 27.12 000 100 0322 0000 076.8 27.16 00005 10010110 17 04 00010 100©w").into_bytes();
        let mut res = String::from("\x00\x00\x00\x00\x00\x00(12345").into_bytes();
        let mut crc_sum = CRCu16::crc16xmodem();
        crc_sum.digest(res.as_slice());
        res.extend_from_slice(crc_sum.get_crc().to_be_bytes().as_ref());
        res.push(b'\r');

        let mut buf = BytesMut::from(res.as_slice());
        let item = codec.decode(&mut buf)?;

        println!("{:?}", item);
        assert_eq!(
            item.unwrap(),
            QIDResponse {
                serial_number: 12345
            }
        );

        Ok(())
    }
}

use crate::command::{Command, Response};
use crate::error::{Error, Result};
use bytes::BytesMut;
use serde_derive::Serialize;

pub struct QVFW2;

impl Command for QVFW2 {
    const PROTOCOL_ID: &'static [u8] = b"QVFW2";
    const COMMAND_NAME: &'static str = "QueryFirmwareVersion2";

    type Request = ();
    type Response = QVFW2Response;
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct QVFW2Response {
    pub major: u64,
    pub minor: u64,
}

impl Response for QVFW2Response {
    fn decode(src: &mut BytesMut) -> Result<Self> {
        if !src.starts_with(b"VERFW2:") {
            return Err(Error::InvalidPayload(None));
        }

        let src = src.split_off(7);

        let idx = src
            .iter()
            .copied()
            .enumerate()
            .find(|(_, x)| *x == b'.')
            .map(|(i, _)| i);
        let idx = if let Some(x) = idx {
            x
        } else {
            return Err(Error::InvalidPayload(None));
        };

        let (version_major, version_minor) = src.split_at(idx);

        let version_major = u64::from_str_radix(std::str::from_utf8(version_major)?, 16)?;
        let version_minor = u64::from_str_radix(std::str::from_utf8(&version_minor[1..])?, 16)?;

        Ok(Self {
            major: version_major,
            minor: version_minor,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::codec::Codec;
    use crate::command::{Command, Request, Response};
    use crate::commands::qvfw2::{QVFW2Response, QVFW2};
    use crate::error::Result;
    use bytes::{Buf, BytesMut};
    use crc_any::CRCu16;
    use rand::prelude::*;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_qvfw2_payload_encode() -> Result<()> {
        let req: <QVFW2 as Command>::Request = ();

        assert_eq!(req.encode()?, None);

        Ok(())
    }

    #[test]
    fn test_qvfw2_payload_decode() -> Result<()> {
        for _ in 0..1000 {
            let n_maj: u64 = random();
            let n_min: u64 = random();

            let res = format!("VERFW2:{:X}.{:X}", n_maj, n_min);

            let mut buf = BytesMut::from(res.as_str());
            let item = <QVFW2 as Command>::Response::decode(&mut buf)?;

            assert_eq!(
                item,
                QVFW2Response {
                    major: n_maj,
                    minor: n_min
                }
            );
        }

        Ok(())
    }

    #[test]
    fn test_qvfw2_command_encode() -> Result<()> {
        let mut codec = Codec::<QVFW2>::new();

        let mut buf = BytesMut::new();
        codec.encode((), &mut buf)?;

        assert_eq!(buf.bytes(), b"QVFW2\xc3\xf5\r");

        Ok(())
    }

    #[test]
    fn test_qvfw2_command_decode() -> Result<()> {
        let mut codec = Codec::<QVFW2>::new();

        for _ in 0..1000 {
            let n_maj: u64 = random();
            let n_min: u64 = random();

            let mut res = format!("(VERFW2:{:X}.{:X}", n_maj, n_min).into_bytes();
            let mut crc_sum = CRCu16::crc16xmodem();
            crc_sum.digest(res.as_slice());
            res.extend_from_slice(crc_sum.get_crc().to_be_bytes().as_ref());
            res.push(b'\r');

            let mut buf = BytesMut::from(res.as_slice());
            let item = codec.decode(&mut buf)?;

            assert_eq!(buf.remaining(), 0);
            assert_eq!(
                item,
                Some(QVFW2Response {
                    major: n_maj,
                    minor: n_min
                })
            );
        }

        Ok(())
    }
}

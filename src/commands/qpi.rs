use crate::command::{Command, Response};
use crate::error::{Error, Result};
use bytes::BytesMut;
use serde_derive::Serialize;
use std::str::FromStr;

pub struct QPI;

impl Command for QPI {
    const PROTOCOL_ID: &'static [u8] = b"QPI";
    const COMMAND_NAME: &'static str = "QueryProtocolId";

    type Request = ();
    type Response = QPIResponse;
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct QPIResponse {
    pub protocol_id: u64,
}

impl Response for QPIResponse {
    fn decode(src: &mut BytesMut) -> Result<Self> {
        if !src.starts_with(b"PI") {
            return Err(Error::InvalidPayload(None));
        }

        Ok(Self {
            protocol_id: u64::from_str(std::str::from_utf8(src[2..].as_ref())?)?,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::codec::Codec;
    use crate::command::{Command, Request, Response};
    use crate::commands::qpi::{QPIResponse, QPI};
    use crate::error::Result;
    use bytes::{Buf, BytesMut};
    use crc_any::CRCu16;
    use rand::prelude::*;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_qpi_payload_encode() -> Result<()> {
        let req: <QPI as Command>::Request = ();

        assert_eq!(req.encode()?, None);

        Ok(())
    }

    #[test]
    fn test_qpi_payload_decode() -> Result<()> {
        for _ in 0..1000 {
            let n: u64 = random();
            let res = format!("PI{}", n);

            let mut buf = BytesMut::from(res.as_str());
            let item = <QPI as Command>::Response::decode(&mut buf)?;

            assert_eq!(item, QPIResponse { protocol_id: n });
        }

        Ok(())
    }

    #[test]
    fn test_qpi_command_encode() -> Result<()> {
        let mut codec = Codec::<QPI>::new();

        let mut buf = BytesMut::new();
        codec.encode((), &mut buf)?;

        assert_eq!(buf.bytes(), b"QPI\xbe\xac\r");

        Ok(())
    }

    #[test]
    fn test_qpi_command_decode() -> Result<()> {
        let mut codec = Codec::<QPI>::new();

        for _ in 0..1000 {
            let n: u64 = random();

            let mut res = format!("(PI{}", n).into_bytes();
            let mut crc_sum = CRCu16::crc16xmodem();
            crc_sum.digest(res.as_slice());
            res.extend_from_slice(crc_sum.get_crc().to_be_bytes().as_ref());
            res.push(b'\r');

            let mut buf = BytesMut::from(res.as_slice());
            let item = codec.decode(&mut buf)?;

            assert_eq!(buf.remaining(), 0);
            assert_eq!(item, Some(QPIResponse { protocol_id: n }));
        }

        Ok(())
    }
}

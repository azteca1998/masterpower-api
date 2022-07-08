use crate::command::{Command, Response};
use crate::error::Result;
use bytes::BytesMut;
use serde_derive::Serialize;
use std::str::FromStr;

pub struct QID;

impl Command for QID {
    const PROTOCOL_ID: &'static [u8] = b"QID";
    const COMMAND_NAME: &'static str = "QuerySerialNumber";

    type Request = ();
    type Response = QIDResponse;
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct QIDResponse {
    pub(crate) serial_number: u64,
}

impl Response for QIDResponse {
    fn decode(src: &mut BytesMut) -> Result<Self> {
        Ok(Self {
            serial_number: u64::from_str(std::str::from_utf8(src.as_ref())?)?,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::codec::Codec;
    use crate::command::{Command, Request, Response};
    use crate::commands::qid::{QIDResponse, QID};
    use crate::error::Result;
    use bytes::{Buf, BytesMut};
    use crc_any::CRCu16;
    use rand::prelude::*;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_qid_payload_encode() -> Result<()> {
        let req: <QID as Command>::Request = ();

        assert_eq!(req.encode()?, None);

        Ok(())
    }

    #[test]
    fn test_qid_payload_decode() -> Result<()> {
        for _ in 0..1000 {
            let n: u64 = random();
            let res = format!("{}", n);

            let mut buf = BytesMut::from(res.as_str());
            let item = <QID as Command>::Response::decode(&mut buf)?;

            assert_eq!(item, QIDResponse { serial_number: n });
        }

        Ok(())
    }

    #[test]
    fn test_qid_command_encode() -> Result<()> {
        let mut codec = Codec::<QID>::new();

        let mut buf = BytesMut::new();
        codec.encode((), &mut buf)?;

        assert_eq!(buf.bytes(), b"QID\xd6\xea\r");

        Ok(())
    }

    #[test]
    fn test_qid_command_decode() -> Result<()> {
        let mut codec = Codec::<QID>::new();

        for _ in 0..1000 {
            let n: u64 = random();

            let mut res = format!("({}", n).into_bytes();
            let mut crc_sum = CRCu16::crc16xmodem();
            crc_sum.digest(res.as_slice());
            res.extend_from_slice(crc_sum.get_crc().to_be_bytes().as_ref());
            res.push(b'\r');

            let mut buf = BytesMut::from(res.as_slice());
            let item = codec.decode(&mut buf)?;

            assert_eq!(buf.remaining(), 0);
            assert_eq!(item, Some(QIDResponse { serial_number: n }));
        }

        Ok(())
    }
}

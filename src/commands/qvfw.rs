use crate::command::{Command, Response};
use crate::error::{Error, Result};
use bytes::BytesMut;
use serde_derive::Serialize;

pub struct QVFW;

impl Command for QVFW {
    const PROTOCOL_ID: &'static [u8] = b"QVFW";
    const COMMAND_NAME: &'static str = "QueryFirmwareVersion";

    type Request = ();
    type Response = QVFWResponse;
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct QVFWResponse {
    pub major: u64,
    pub minor: u64,
}

impl Response for QVFWResponse {
    fn decode(src: &mut BytesMut) -> Result<Self> {
        if !src.starts_with(b"VERFW:") {
            return Err(Error::InvalidPayload(None));
        }

        let src = src.split_off(6);

        let idx = src
            .iter()
            .copied()
            .enumerate()
            .find(|(_, x)| *x == '.' as u8)
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
    use crate::commands::qvfw::{QVFWResponse, QVFW};
    use crate::error::Result;
    use bytes::{Buf, BytesMut};
    use crc_any::CRCu16;
    use rand::prelude::*;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_qvfw_payload_encode() -> Result<()> {
        let req: <QVFW as Command>::Request = ();

        assert_eq!(req.encode()?, None);

        Ok(())
    }

    #[test]
    fn test_qvfw_payload_decode() -> Result<()> {
        for _ in 0..1000 {
            let n_maj: u64 = random();
            let n_min: u64 = random();

            let res = format!("VERFW:{:X}.{:X}", n_maj, n_min);

            let mut buf = BytesMut::from(res.as_str());
            let item = <QVFW as Command>::Response::decode(&mut buf)?;

            assert_eq!(
                item,
                QVFWResponse {
                    major: n_maj,
                    minor: n_min
                }
            );
        }

        Ok(())
    }

    #[test]
    fn test_qvfw_command_encode() -> Result<()> {
        let mut codec = Codec::<QVFW>::new();

        let mut buf = BytesMut::new();
        codec.encode((), &mut buf)?;

        assert_eq!(buf.bytes(), b"QVFW\x62\x99\r");

        Ok(())
    }

    #[test]
    fn test_qvfw_command_decode() -> Result<()> {
        let mut codec = Codec::<QVFW>::new();

        for _ in 0..1000 {
            let n_maj: u64 = random();
            let n_min: u64 = random();

            let mut res = format!("(VERFW:{:X}.{:X}", n_maj, n_min).into_bytes();
            let mut crc_sum = CRCu16::crc16xmodem();
            crc_sum.digest(res.as_slice());
            res.extend_from_slice(crc_sum.get_crc().to_be_bytes().as_ref());
            res.push(b'\r');

            let mut buf = BytesMut::from(res.as_slice());
            let item = codec.decode(&mut buf)?;

            assert_eq!(buf.remaining(), 0);
            assert_eq!(
                item,
                Some(QVFWResponse {
                    major: n_maj,
                    minor: n_min
                })
            );
        }

        Ok(())
    }
}

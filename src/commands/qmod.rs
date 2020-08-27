use crate::command::{Command, Response};
use crate::commands::qmod::DeviceMode::{
    BatteryMode, FaultMode, LineMode, PowerOnMode, PowerSavingMode, StandbyMode,
};
use crate::error::{Error, Result};
use bytes::BytesMut;
use serde_derive::Serialize;
use std::str::from_utf8;

pub struct QMOD;

impl Command for QMOD {
    const PROTOCOL_ID: &'static [u8] = b"QMOD";
    const COMMAND_NAME: &'static str = "DeviceModeInquiry";

    type Request = ();
    type Response = QMODResponse;
}

#[derive(Debug, PartialEq, Serialize)]
pub struct QMODResponse {
    pub(crate) mode: DeviceMode,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum DeviceMode {
    PowerOnMode,
    StandbyMode,
    LineMode,
    BatteryMode,
    FaultMode,
    PowerSavingMode,
}

impl Response for QMODResponse {
    fn decode(src: &mut BytesMut) -> Result<Self> {
        Ok(Self {
            mode: match from_utf8(&src[0..1])? {
                "P" => PowerOnMode,
                "S" => StandbyMode,
                "L" => LineMode,
                "B" => BatteryMode,
                "F" => FaultMode,
                "H" => PowerSavingMode,
                _ => return Err(Error::InvalidDeviceMode),
            },
        })
    }
}

#[cfg(test)]
mod test {
    use crate::codec::Codec;
    use crate::command::{Command, Request, Response};
    use crate::commands::qmod::DeviceMode::{
        BatteryMode, FaultMode, LineMode, PowerOnMode, PowerSavingMode, StandbyMode,
    };
    use crate::commands::qmod::{QMODResponse, QMOD};
    use crate::error::Result;
    use bytes::{Buf, BytesMut};
    use crc_any::CRCu16;
    use rand::{thread_rng, Rng};
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_qmod_payload_encode() -> Result<()> {
        let req: <QMOD as Command>::Request = ();

        assert_eq!(req.encode()?, None);

        Ok(())
    }

    #[test]
    fn test_qmod_payload_decode() -> Result<()> {
        let device_mode_options = ["P", "S", "L", "B", "F", "H"];

        for _ in 0..1000 {
            let mut rng = thread_rng();
            let mode = device_mode_options[rng.gen_range(0, device_mode_options.len())];

            let mut buf = BytesMut::from(mode);
            let item = <QMOD as Command>::Response::decode(&mut buf)?;

            assert_eq!(
                item,
                QMODResponse {
                    mode: match mode {
                        "P" => PowerOnMode,
                        "S" => StandbyMode,
                        "L" => LineMode,
                        "B" => BatteryMode,
                        "F" => FaultMode,
                        "H" => PowerSavingMode,
                        _ => unreachable!(),
                    },
                }
            );
        }

        Ok(())
    }

    #[test]
    fn test_qmod_command_encode() -> Result<()> {
        let mut codec = Codec::<QMOD>::new();

        let mut buf = BytesMut::new();
        codec.encode((), &mut buf)?;

        assert_eq!(buf.bytes(), b"QMOD\x49\xc1\r");

        Ok(())
    }

    #[test]
    fn test_qmod_command_decode() -> Result<()> {
        let mut codec = Codec::<QMOD>::new();
        let device_mode_options = ["P", "S", "L", "B", "F", "H"];

        for _ in 0..1000 {
            let mut rng = thread_rng();

            let mode = device_mode_options[rng.gen_range(0, device_mode_options.len())];
            let mut res = format!("({}", mode).into_bytes();
            let mut crc_sum = CRCu16::crc16xmodem();
            crc_sum.digest(res.as_slice());
            res.extend_from_slice(crc_sum.get_crc().to_be_bytes().as_ref());
            res.push(b'\r');

            let mut buf = BytesMut::from(res.as_slice());
            let item = codec.decode(&mut buf)?;

            assert_eq!(buf.remaining(), 0);
            assert_eq!(
                item,
                Some(QMODResponse {
                    mode: match mode {
                        "P" => PowerOnMode,
                        "S" => StandbyMode,
                        "L" => LineMode,
                        "B" => BatteryMode,
                        "F" => FaultMode,
                        "H" => PowerSavingMode,
                        _ => unreachable!(),
                    },
                })
            );
        }

        Ok(())
    }
}

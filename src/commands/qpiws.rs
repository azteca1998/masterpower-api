use crate::command::{Command, Response};
use crate::error::{Error, Result};
use bytes::BytesMut;
use serde_derive::Serialize;
use std::str::from_utf8;

pub struct QPIWS;

impl Command for QPIWS {
    const PROTOCOL_ID: &'static [u8] = b"QPIWS";
    const COMMAND_NAME: &'static str = "DeviceWarningStatus";

    type Request = ();
    type Response = QPIWSResponse;
}

#[derive(Debug, PartialEq, Serialize)]
pub struct QPIWSResponse {
    pub inverter_fault: bool,
    pub bus_over: bool,
    pub bus_under: bool,
    pub bus_soft_fail: bool,
    pub line_fail: bool,
    pub opv_short: bool,
    pub inverter_voltage_too_low: bool,
    pub inverter_voltage_too_high: bool,
    pub over_temperature: bool,
    pub fan_locked: bool,
    pub battery_voltage_high: bool,
    pub battery_low_alarm: bool,
    pub battery_under_shutdown: bool,
    pub over_load: bool,
    pub eeprom_fault: bool,
    pub inverter_over_current: bool,
    pub inverter_soft_fail: bool,
    pub self_test_fail: bool,
    pub op_dc_voltage_over: bool,
    pub bat_open: bool,
    pub current_sensor_fail: bool,
    pub battery_short: bool,
    pub power_limit: bool,
    pub pv_voltage_high: bool,
    pub mppt_overload_fault: bool,
    pub mppt_overload_warning: bool,
    pub battery_too_low_to_charge: bool,
}

impl QPIWSResponse {
    fn decode_warning(src: &mut BytesMut, position: usize) -> Result<bool> {
        Ok(match from_utf8(&src[position..(position + 1)])? {
            "0" => false,
            "1" => true,
            _ => return Err(Error::InvalidWarningStatus),
        })
    }
}

impl Response for QPIWSResponse {
    fn decode(src: &mut BytesMut) -> Result<Self> {
        Ok(Self {
            inverter_fault: Self::decode_warning(src, 1)?,
            bus_over: Self::decode_warning(src, 2)?,
            bus_under: Self::decode_warning(src, 3)?,
            bus_soft_fail: Self::decode_warning(src, 4)?,
            line_fail: Self::decode_warning(src, 5)?,
            opv_short: Self::decode_warning(src, 6)?,
            inverter_voltage_too_low: Self::decode_warning(src, 7)?,
            inverter_voltage_too_high: Self::decode_warning(src, 8)?,
            over_temperature: Self::decode_warning(src, 9)?,
            fan_locked: Self::decode_warning(src, 10)?,
            battery_voltage_high: Self::decode_warning(src, 11)?,
            battery_low_alarm: Self::decode_warning(src, 12)?,
            battery_under_shutdown: Self::decode_warning(src, 14)?,
            over_load: Self::decode_warning(src, 16)?,
            eeprom_fault: Self::decode_warning(src, 17)?,
            inverter_over_current: Self::decode_warning(src, 18)?,
            inverter_soft_fail: Self::decode_warning(src, 19)?,
            self_test_fail: Self::decode_warning(src, 20)?,
            op_dc_voltage_over: Self::decode_warning(src, 21)?,
            bat_open: Self::decode_warning(src, 22)?,
            current_sensor_fail: Self::decode_warning(src, 23)?,
            battery_short: Self::decode_warning(src, 24)?,
            power_limit: Self::decode_warning(src, 25)?,
            pv_voltage_high: Self::decode_warning(src, 26)?,
            mppt_overload_fault: Self::decode_warning(src, 27)?,
            mppt_overload_warning: Self::decode_warning(src, 28)?,
            battery_too_low_to_charge: Self::decode_warning(src, 29)?,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::codec::Codec;
    use crate::command::{Command, Request, Response};
    use crate::commands::qpiws::{QPIWSResponse, QPIWS};
    use crate::error::Result;
    use bytes::{Buf, BytesMut};
    use crc_any::CRCu16;
    use rand::{thread_rng, Rng};
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_qpiws_payload_encode() -> Result<()> {
        let req: <QPIWS as Command>::Request = ();

        assert_eq!(req.encode()?, None);

        Ok(())
    }

    #[test]
    fn test_qpiws_payload_decode() -> Result<()> {
        for _ in 0..1000 {
            let mut rng = thread_rng();

            let inverter_fault = rng.gen_bool(0.5f64);
            let bus_over = rng.gen_bool(0.5f64);
            let bus_under = rng.gen_bool(0.5f64);
            let bus_soft_fail = rng.gen_bool(0.5f64);
            let line_fail = rng.gen_bool(0.5f64);
            let opv_short = rng.gen_bool(0.5f64);
            let inverter_voltage_too_low = rng.gen_bool(0.5f64);
            let inverter_voltage_too_high = rng.gen_bool(0.5f64);
            let over_temperature = rng.gen_bool(0.5f64);
            let fan_locked = rng.gen_bool(0.5f64);
            let battery_voltage_high = rng.gen_bool(0.5f64);
            let battery_low_alarm = rng.gen_bool(0.5f64);
            let battery_under_shutdown = rng.gen_bool(0.5f64);
            let over_load = rng.gen_bool(0.5f64);
            let eeprom_fault = rng.gen_bool(0.5f64);
            let inverter_over_current = rng.gen_bool(0.5f64);
            let inverter_soft_fail = rng.gen_bool(0.5f64);
            let self_test_fail = rng.gen_bool(0.5f64);
            let op_dc_voltage_over = rng.gen_bool(0.5f64);
            let bat_open = rng.gen_bool(0.5f64);
            let current_sensor_fail = rng.gen_bool(0.5f64);
            let battery_short = rng.gen_bool(0.5f64);
            let power_limit = rng.gen_bool(0.5f64);
            let pv_voltage_high = rng.gen_bool(0.5f64);
            let mppt_overload_fault = rng.gen_bool(0.5f64);
            let mppt_overload_warning = rng.gen_bool(0.5f64);
            let battery_too_low_to_charge = rng.gen_bool(0.5f64);

            let mut buf = BytesMut::from(
                format!(
                    "0{}{}{}{}{}{}{}{}{}{}{}{}0{}0{}{}{}{}{}{}{}{}{}{}{}{}{}{}00",
                    inverter_fault as i32,
                    bus_over as i32,
                    bus_under as i32,
                    bus_soft_fail as i32,
                    line_fail as i32,
                    opv_short as i32,
                    inverter_voltage_too_low as i32,
                    inverter_voltage_too_high as i32,
                    over_temperature as i32,
                    fan_locked as i32,
                    battery_voltage_high as i32,
                    battery_low_alarm as i32,
                    battery_under_shutdown as i32,
                    over_load as i32,
                    eeprom_fault as i32,
                    inverter_over_current as i32,
                    inverter_soft_fail as i32,
                    self_test_fail as i32,
                    op_dc_voltage_over as i32,
                    bat_open as i32,
                    current_sensor_fail as i32,
                    battery_short as i32,
                    power_limit as i32,
                    pv_voltage_high as i32,
                    mppt_overload_fault as i32,
                    mppt_overload_warning as i32,
                    battery_too_low_to_charge as i32,
                )
                .into_bytes()
                .as_slice(),
            );
            let item = <QPIWS as Command>::Response::decode(&mut buf)?;

            assert_eq!(
                item,
                QPIWSResponse {
                    inverter_fault,
                    bus_over,
                    bus_under,
                    bus_soft_fail,
                    line_fail,
                    opv_short,
                    inverter_voltage_too_low,
                    inverter_voltage_too_high,
                    over_temperature,
                    fan_locked,
                    battery_voltage_high,
                    battery_low_alarm,
                    battery_under_shutdown,
                    over_load,
                    eeprom_fault,
                    inverter_over_current,
                    inverter_soft_fail,
                    self_test_fail,
                    op_dc_voltage_over,
                    bat_open,
                    current_sensor_fail,
                    battery_short,
                    power_limit,
                    pv_voltage_high,
                    mppt_overload_fault,
                    mppt_overload_warning,
                    battery_too_low_to_charge
                }
            );
        }

        Ok(())
    }

    #[test]
    fn test_qpiws_command_encode() -> Result<()> {
        let mut codec = Codec::<QPIWS>::new();

        let mut buf = BytesMut::new();
        codec.encode((), &mut buf)?;

        assert_eq!(buf.bytes(), b"QPIWS\xb4\xda\r");

        Ok(())
    }

    #[test]
    fn test_qpiws_command_decode() -> Result<()> {
        let mut codec = Codec::<QPIWS>::new();

        for _ in 0..1000 {
            let mut rng = thread_rng();

            let inverter_fault = rng.gen_bool(0.5f64);
            let bus_over = rng.gen_bool(0.5f64);
            let bus_under = rng.gen_bool(0.5f64);
            let bus_soft_fail = rng.gen_bool(0.5f64);
            let line_fail = rng.gen_bool(0.5f64);
            let opv_short = rng.gen_bool(0.5f64);
            let inverter_voltage_too_low = rng.gen_bool(0.5f64);
            let inverter_voltage_too_high = rng.gen_bool(0.5f64);
            let over_temperature = rng.gen_bool(0.5f64);
            let fan_locked = rng.gen_bool(0.5f64);
            let battery_voltage_high = rng.gen_bool(0.5f64);
            let battery_low_alarm = rng.gen_bool(0.5f64);
            let battery_under_shutdown = rng.gen_bool(0.5f64);
            let over_load = rng.gen_bool(0.5f64);
            let eeprom_fault = rng.gen_bool(0.5f64);
            let inverter_over_current = rng.gen_bool(0.5f64);
            let inverter_soft_fail = rng.gen_bool(0.5f64);
            let self_test_fail = rng.gen_bool(0.5f64);
            let op_dc_voltage_over = rng.gen_bool(0.5f64);
            let bat_open = rng.gen_bool(0.5f64);
            let current_sensor_fail = rng.gen_bool(0.5f64);
            let battery_short = rng.gen_bool(0.5f64);
            let power_limit = rng.gen_bool(0.5f64);
            let pv_voltage_high = rng.gen_bool(0.5f64);
            let mppt_overload_fault = rng.gen_bool(0.5f64);
            let mppt_overload_warning = rng.gen_bool(0.5f64);
            let battery_too_low_to_charge = rng.gen_bool(0.5f64);

            let mut res = format!(
                "(0{}{}{}{}{}{}{}{}{}{}{}{}0{}0{}{}{}{}{}{}{}{}{}{}{}{}{}{}00",
                inverter_fault as i32,
                bus_over as i32,
                bus_under as i32,
                bus_soft_fail as i32,
                line_fail as i32,
                opv_short as i32,
                inverter_voltage_too_low as i32,
                inverter_voltage_too_high as i32,
                over_temperature as i32,
                fan_locked as i32,
                battery_voltage_high as i32,
                battery_low_alarm as i32,
                battery_under_shutdown as i32,
                over_load as i32,
                eeprom_fault as i32,
                inverter_over_current as i32,
                inverter_soft_fail as i32,
                self_test_fail as i32,
                op_dc_voltage_over as i32,
                bat_open as i32,
                current_sensor_fail as i32,
                battery_short as i32,
                power_limit as i32,
                pv_voltage_high as i32,
                mppt_overload_fault as i32,
                mppt_overload_warning as i32,
                battery_too_low_to_charge as i32,
            )
            .into_bytes();
            let mut crc_sum = CRCu16::crc16xmodem();
            crc_sum.digest(res.as_slice());
            res.extend_from_slice(crc_sum.get_crc().to_be_bytes().as_ref());
            res.push(b'\r');

            let mut buf = BytesMut::from(res.as_slice());
            let item = codec.decode(&mut buf)?;

            assert_eq!(buf.remaining(), 0);
            assert_eq!(
                item,
                Some(QPIWSResponse {
                    inverter_fault,
                    bus_over,
                    bus_under,
                    bus_soft_fail,
                    line_fail,
                    opv_short,
                    inverter_voltage_too_low,
                    inverter_voltage_too_high,
                    over_temperature,
                    fan_locked,
                    battery_voltage_high,
                    battery_low_alarm,
                    battery_under_shutdown,
                    over_load,
                    eeprom_fault,
                    inverter_over_current,
                    inverter_soft_fail,
                    self_test_fail,
                    op_dc_voltage_over,
                    bat_open,
                    current_sensor_fail,
                    battery_short,
                    power_limit,
                    pv_voltage_high,
                    mppt_overload_fault,
                    mppt_overload_warning,
                    battery_too_low_to_charge
                })
            );
        }

        Ok(())
    }
}

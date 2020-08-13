use crate::command::{Command, Response};
use crate::commands::qpigs::DeviceChargingStatus::{
    ChargingFromAC, ChargingFromSCC, ChargingFromSCCAndAC, NotCharging,
};
use crate::error::Result;
use bytes::BytesMut;
use std::str::from_utf8;
use std::str::FromStr;

pub struct QPIGS;

impl Command for QPIGS {
    const PROTOCOL_ID: &'static [u8] = b"QPIGS";
    const COMMAND_NAME: &'static str = "QueryDeviceGeneralStatus";

    type Request = ();
    type Response = QPIGSResponse;
}

#[derive(Debug, PartialEq)]
pub struct QPIGSResponse {
    pub grid_voltage: f32,
    pub grid_frequency: f32,
    pub ac_out_voltage: f32,
    pub ac_out_frequency: f32,
    pub ac_out_apparent_power: usize,
    pub ac_out_active_power: usize,
    pub out_load_percent: usize,
    pub bus_voltage: usize,
    pub battery_voltage: f32,
    pub battery_charge_current: usize,
    pub battery_capacity: usize,
    pub inverter_heat_sink_temp: usize,
    pub pv_input_current: usize,
    pub pv_input_voltage: f32,
    pub battery_scc_voltage: f32,
    pub battery_discharge_current: usize,
    pub device_status: DeviceStatus,
}

#[derive(Debug, PartialEq)]
pub struct DeviceStatus {
    charge_status: DeviceChargingStatus,
    active_load: bool,
}

#[derive(Debug, PartialEq)]
pub enum DeviceChargingStatus {
    NotCharging,
    ChargingFromSCC,
    ChargingFromAC,
    ChargingFromSCCAndAC,
}

impl Response for QPIGSResponse {
    fn decode(src: &mut BytesMut) -> Result<Self> {
        // println!("Input: {:?}", from_utf8(&src)?);

        // Extract indices
        let mut idxs = [0usize; 22];
        src.iter()
            .cloned()
            .enumerate()
            .filter(|(_, x)| *x == ' ' as u8)
            .fold(0, |num_idx, byte_idx| {
                idxs[num_idx] = byte_idx.0;
                num_idx + 1
            });

        // Extract data
        let grid_voltage = f32::from_str(from_utf8(&src[0..idxs[0]])?)?;
        let grid_frequency = f32::from_str(from_utf8(&src[idxs[0] + 1..idxs[1]])?)?;
        let ac_out_voltage = f32::from_str(from_utf8(&src[idxs[1] + 1..idxs[2]])?)?;
        let ac_out_frequency = f32::from_str(from_utf8(&src[idxs[2] + 1..idxs[3]])?)?;
        let ac_out_apparent_power = usize::from_str(from_utf8(&src[idxs[3] + 1..idxs[4]])?)?;
        let ac_out_active_power = usize::from_str(from_utf8(&src[idxs[4] + 1..idxs[5]])?)?;
        let out_load_percent = usize::from_str(from_utf8(&src[idxs[5] + 1..idxs[6]])?)?;
        let bus_voltage = usize::from_str(from_utf8(&src[idxs[6] + 1..idxs[7]])?)?;
        let battery_voltage = f32::from_str(from_utf8(&src[idxs[7] + 1..idxs[8]])?)?;
        let battery_charge_current = usize::from_str(from_utf8(&src[idxs[8] + 1..idxs[9]])?)?;
        let battery_capacity = usize::from_str(from_utf8(&src[idxs[9] + 1..idxs[10]])?)?;
        let inverter_heat_sink_temp = usize::from_str(from_utf8(&src[idxs[10] + 1..idxs[11]])?)?;
        let pv_input_current = usize::from_str(from_utf8(&src[idxs[11] + 1..idxs[12]])?)?;
        let pv_input_voltage = f32::from_str(from_utf8(&src[idxs[12] + 1..idxs[13]])?)?;
        let battery_scc_voltage = f32::from_str(from_utf8(&src[idxs[13] + 1..idxs[14]])?)?;
        let battery_discharge_current = usize::from_str(from_utf8(&src[idxs[14] + 1..idxs[15]])?)?;
        let device_status = &src[idxs[15] + 1..idxs[16]];

        // println!("Remaining: {:?}", &src[idxs[16] + 1..]);

        Ok(Self {
            grid_voltage,
            grid_frequency,
            ac_out_voltage,
            ac_out_frequency,
            ac_out_apparent_power,
            ac_out_active_power,
            out_load_percent,
            bus_voltage,
            battery_voltage,
            battery_charge_current,
            battery_capacity,
            inverter_heat_sink_temp,
            pv_input_current,
            pv_input_voltage,
            battery_scc_voltage,
            battery_discharge_current,
            device_status: DeviceStatus {
                active_load: from_utf8(&device_status[3..4])? == "1",
                charge_status: match from_utf8(&device_status[5..8])? {
                    "000" => NotCharging,
                    "110" => ChargingFromSCC,
                    "101" => ChargingFromAC,
                    "111" => ChargingFromSCCAndAC,
                    _ => unimplemented!(),
                },
            },
        })
    }
}

#[cfg(test)]
mod test {
    use crate::codec::Codec;
    use crate::command::{Command, Request, Response};
    use crate::commands::qpigs::DeviceChargingStatus::{
        ChargingFromAC, ChargingFromSCC, ChargingFromSCCAndAC, NotCharging,
    };
    use crate::commands::qpigs::{DeviceStatus, QPIGSResponse, QPIGS};
    use crate::error::Result;
    use bytes::{Buf, BytesMut};
    use crc_any::CRCu16;
    use rand::{random, thread_rng, Rng};
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_qpigs_payload_encode() -> Result<()> {
        let req: <QPIGS as Command>::Request = ();
        assert_eq!(req.encode()?, None);
        Ok(())
    }

    #[test]
    fn test_qpigs_payload_decode_custom() -> Result<()> {
        let res = "001.0 00.0 229.0 50.0 0091 0091 003 420 27.16 000 100 0336 0000 074.9 27.12 00005 10110110 17 04 00010 100";

        let mut buf = BytesMut::from(res);
        let item = <QPIGS as Command>::Response::decode(&mut buf)?;
        // println!("Test result: {:#?}", item);
        assert_eq!(
            item,
            QPIGSResponse {
                grid_voltage: 1.0f32,
                grid_frequency: 0.0f32,
                ac_out_voltage: 229.0f32,
                ac_out_frequency: 50.0f32,
                ac_out_apparent_power: 91,
                ac_out_active_power: 91,
                out_load_percent: 3,
                bus_voltage: 420,
                battery_voltage: 27.16f32,
                battery_charge_current: 0,
                battery_capacity: 100,
                inverter_heat_sink_temp: 336,
                pv_input_current: 0,
                pv_input_voltage: 74.9f32,
                battery_scc_voltage: 27.12f32,
                battery_discharge_current: 5,
                device_status: DeviceStatus {
                    active_load: true,
                    charge_status: ChargingFromSCC,
                },
            }
        );

        Ok(())
    }

    #[test]
    fn test_qpigs_command_encode() -> Result<()> {
        let mut codec = Codec::<QPIGS>::new();

        let mut buf = BytesMut::new();
        codec.encode((), &mut buf)?;

        assert_eq!(buf.bytes(), b"QPIGS\xb7\xa9\r");

        Ok(())
    }

    #[test]
    fn test_qpigs_command_decode() -> Result<()> {
        let mut codec = Codec::<QPIGS>::new();

        let device_status_options = ["000", "110", "101", "111"];

        for _ in 0..1000 {
            let mut rng = thread_rng();
            let grid_voltage: f32 = (rng.gen_range(0.0, 500.0) * 10.0f32).floor() / 10.0;
            let grid_frequency: f32 = (rng.gen_range(0.0, 60.0) * 10.0f32).floor() / 10.0;
            let ac_out_voltage: f32 = (rng.gen_range(0.0, 500.0) * 10.0f32).floor() / 10.0;
            let ac_out_frequency: f32 = (rng.gen_range(0.0, 60.0) * 10.0f32).floor() / 10.0;
            let ac_out_apparent_power: usize = rng.gen_range(0, 2500);
            let ac_out_active_power: usize = rng.gen_range(0, 2500);
            let out_load_percent: usize = rng.gen_range(0, 100);
            let bus_voltage: usize = rng.gen_range(0, 2500);
            let battery_voltage: f32 = (rng.gen_range(0.0, 100.0) * 10.0f32).floor() / 10.0;
            let battery_charge_current: usize = rng.gen_range(0, 2500);
            let battery_capacity: usize = rng.gen_range(0, 100);
            let inverter_heat_sink_temp: usize = rng.gen_range(0, 100);
            let pv_input_current: usize = rng.gen_range(0, 100);
            let pv_input_voltage: f32 = (rng.gen_range(0.0, 100.0) * 10.0f32).floor() / 10.0;
            let battery_scc_voltage: f32 = (rng.gen_range(0.0, 100.0) * 10.0f32).floor() / 10.0;
            let battery_discharge_current: usize = rng.gen_range(0, 100);

            let active_load: bool = random();
            let charge_status =
                device_status_options[rng.gen_range(0, device_status_options.len())];

            let mut res = format!(
                // 001.0   00.0    229.0   50.0   0091   0091  003  420    27.16   000   100   0336 0000  074.9   27.12   00005 10110110 17 04 00010 100
                "({:03.1} {:02.1} {:03.1} {:02.1} {:04} {:04} {:03} {:03} {:02.2} {:03} {:03} {:04} {:04} {:03.1} {:02.2} {:05} 000{:b}0{} 17 04 00010 100",
                grid_voltage,
                grid_frequency,
                ac_out_voltage,
                ac_out_frequency,
                ac_out_apparent_power,
                ac_out_active_power,
                out_load_percent,
                bus_voltage,
                battery_voltage,
                battery_charge_current,
                battery_capacity,
                inverter_heat_sink_temp,
                pv_input_current,
                pv_input_voltage,
                battery_scc_voltage,
                battery_discharge_current,
                active_load as i32,
                charge_status
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
                Some(QPIGSResponse {
                    grid_voltage,
                    grid_frequency,
                    ac_out_voltage,
                    ac_out_frequency,
                    ac_out_apparent_power,
                    ac_out_active_power,
                    out_load_percent,
                    bus_voltage,
                    battery_voltage,
                    battery_charge_current,
                    battery_capacity,
                    inverter_heat_sink_temp,
                    pv_input_current,
                    pv_input_voltage,
                    battery_scc_voltage,
                    battery_discharge_current,
                    device_status: DeviceStatus {
                        active_load: active_load,
                        charge_status: match charge_status {
                            "000" => NotCharging,
                            "110" => ChargingFromSCC,
                            "101" => ChargingFromAC,
                            "111" => ChargingFromSCCAndAC,
                            _ => unimplemented!(),
                        },
                    },
                })
            );
        }

        Ok(())
    }
}

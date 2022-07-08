use crate::command::{Command, Response};
use crate::commands::qpiri::BatteryType::{Flooded, User, AGM};
use crate::commands::qpiri::ChargeSourcePriority::{
    GridFirst as ChargeSourceGridFirst, OnlySolar, SolarAndGrid,
    SolarFirst as ChargeSourceSolarFirst,
};
use crate::commands::qpiri::InputVoltageRange::{Appliance, UPS};
use crate::commands::qpiri::MachineType::{GridTie, Hybrid, OffGrid};
use crate::commands::qpiri::OutputMode::{
    ParallelOutput, Phase1Of3Output, Phase2Of3Output, Phase3Of3Output, SingleMachineOutput,
};
use crate::commands::qpiri::OutputSourcePriority::{
    GridFirst as OutputSourceGridFirst, SBUFirst, SolarFirst as OutputSourceSolarFirst,
};
use crate::commands::qpiri::Topology::{Transformer, Transformerless};
use crate::error::{Error, Result};
use bytes::BytesMut;
use serde_derive::Serialize;
use std::str::from_utf8;
use std::str::FromStr;

pub struct QPIRI;

impl Command for QPIRI {
    const PROTOCOL_ID: &'static [u8] = b"QPIRI";
    const COMMAND_NAME: &'static str = "QueryDeviceRatingInformation";

    type Request = ();
    type Response = QPIRIResponse;
}

#[derive(Debug, PartialEq, Serialize)]
pub struct QPIRIResponse {
    pub grid_rating_voltage: f32,
    pub grid_rating_current: f32,
    pub ac_output_rating_voltage: f32,
    pub ac_out_rating_frequency: f32,
    pub ac_out_rating_current: f32,
    pub ac_out_rating_apparent_power: i32,
    pub ac_out_rating_active_power: i32,
    pub battery_rating_voltage: f32,
    pub battery_recharge_voltage: f32,
    pub battery_under_voltage: f32,
    pub battery_bulk_voltage: f32,
    pub battery_float_voltage: f32,
    pub battery_type: BatteryType,
    pub max_ac_charging_current: i32,
    pub max_charging_current: i32,
    pub input_voltage_range: InputVoltageRange,
    pub output_source_priority: OutputSourcePriority,
    pub charge_source_priority: ChargeSourcePriority,
    pub machine_type: MachineType,
    pub topology: Topology,
    pub output_mode: OutputMode,
    pub battery_redischarge_voltage: f32,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum BatteryType {
    AGM,
    Flooded,
    User,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum InputVoltageRange {
    Appliance,
    UPS,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum OutputSourcePriority {
    GridFirst,
    SolarFirst,
    SBUFirst,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum ChargeSourcePriority {
    GridFirst,
    SolarFirst,
    SolarAndGrid,
    OnlySolar,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum MachineType {
    GridTie,
    OffGrid,
    Hybrid,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum Topology {
    Transformerless,
    Transformer,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum OutputMode {
    SingleMachineOutput,
    ParallelOutput,
    Phase1Of3Output,
    Phase2Of3Output,
    Phase3Of3Output,
}

impl Response for QPIRIResponse {
    fn decode(src: &mut BytesMut) -> Result<Self> {
        // println!("Input: {:?}", from_utf8(&src)?);

        // Extract indices
        let mut idxs = [0usize; 24];
        src.iter()
            .cloned()
            .enumerate()
            .filter(|(_, x)| *x == ' ' as u8)
            .fold(0, |num_idx, byte_idx| {
                idxs[num_idx] = byte_idx.0;
                num_idx + 1
            });

        // Extract data
        let grid_rating_voltage = f32::from_str(from_utf8(&src[0..idxs[0]])?)?;
        let grid_rating_current = f32::from_str(from_utf8(&src[idxs[0] + 1..idxs[1]])?)?;
        let ac_output_rating_voltage = f32::from_str(from_utf8(&src[idxs[1] + 1..idxs[2]])?)?;
        let ac_out_rating_frequency = f32::from_str(from_utf8(&src[idxs[2] + 1..idxs[3]])?)?;
        let ac_out_rating_current = f32::from_str(from_utf8(&src[idxs[3] + 1..idxs[4]])?)?;
        let ac_out_rating_apparent_power = i32::from_str(from_utf8(&src[idxs[4] + 1..idxs[5]])?)?;
        let ac_out_rating_active_power = i32::from_str(from_utf8(&src[idxs[5] + 1..idxs[6]])?)?;
        let battery_rating_voltage = f32::from_str(from_utf8(&src[idxs[6] + 1..idxs[7]])?)?;
        let battery_recharge_voltage = f32::from_str(from_utf8(&src[idxs[7] + 1..idxs[8]])?)?;
        let battery_under_voltage = f32::from_str(from_utf8(&src[idxs[8] + 1..idxs[9]])?)?;
        let battery_bulk_voltage = f32::from_str(from_utf8(&src[idxs[9] + 1..idxs[10]])?)?;
        let battery_float_voltage = f32::from_str(from_utf8(&src[idxs[10] + 1..idxs[11]])?)?;
        let battery_type = from_utf8(&src[idxs[11] + 1..idxs[12]])?;
        let max_ac_charging_current = i32::from_str(from_utf8(&src[idxs[12] + 1..idxs[13]])?)?;
        let max_charging_current = i32::from_str(from_utf8(&src[idxs[13] + 1..idxs[14]])?)?;
        let input_voltage_range = from_utf8(&src[idxs[14] + 1..idxs[15]])?;
        let output_source_priority = from_utf8(&src[idxs[15] + 1..idxs[16]])?;
        let charge_source_priority = from_utf8(&src[idxs[16] + 1..idxs[17]])?;
        let machine_type = from_utf8(&src[idxs[18] + 1..idxs[19]])?;
        let topology = from_utf8(&src[idxs[19] + 1..idxs[20]])?;
        let output_mode = from_utf8(&src[idxs[20] + 1..idxs[21]])?;
        let battery_redischarge_voltage = f32::from_str(from_utf8(&src[idxs[21] + 1..idxs[22]])?)?;

        Ok(Self {
            grid_rating_voltage,
            grid_rating_current,
            ac_output_rating_voltage,
            ac_out_rating_frequency,
            ac_out_rating_current,
            ac_out_rating_apparent_power,
            ac_out_rating_active_power,
            battery_rating_voltage,
            battery_recharge_voltage,
            battery_under_voltage,
            battery_bulk_voltage,
            battery_float_voltage,
            battery_type: match battery_type {
                "0" => AGM,
                "1" => Flooded,
                "2" => User,
                _ => return Err(Error::InvalidDeviceBatteryType),
            },
            max_ac_charging_current,
            max_charging_current,
            input_voltage_range: match input_voltage_range {
                "0" => Appliance,
                "1" => UPS,
                _ => return Err(Error::InvalidDeviceInputVoltageRange),
            },
            output_source_priority: match output_source_priority {
                "0" => OutputSourceGridFirst,
                "1" => OutputSourceSolarFirst,
                "2" => SBUFirst,
                _ => return Err(Error::InvalidDeviceOutputSourcePriority),
            },
            charge_source_priority: match charge_source_priority {
                "0" => ChargeSourceGridFirst,
                "1" => ChargeSourceSolarFirst,
                "2" => SolarAndGrid,
                "3" => OnlySolar,
                _ => return Err(Error::InvalidDeviceChargeSourcePriority),
            },
            machine_type: match machine_type {
                "00" => GridTie,
                "01" => OffGrid,
                "10" => Hybrid,
                _ => return Err(Error::InvalidDeviceMachineType),
            },
            topology: match topology {
                "0" => Transformerless,
                "1" => Transformer,
                _ => return Err(Error::InvalidDeviceTopology),
            },
            output_mode: match output_mode {
                "0" => SingleMachineOutput,
                "1" => ParallelOutput,
                "2" => Phase1Of3Output,
                "3" => Phase2Of3Output,
                "4" => Phase3Of3Output,
                _ => return Err(Error::InvalidDeviceOutputMode),
            },
            battery_redischarge_voltage,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::codec::Codec;
    use crate::command::{Command, Request, Response};
    use crate::commands::qpiri::BatteryType::{Flooded, User, AGM};
    use crate::commands::qpiri::ChargeSourcePriority::{
        GridFirst as ChargeSourceGridFirst, OnlySolar, SolarAndGrid,
        SolarFirst as ChargeSourceSolarFirst,
    };
    use crate::commands::qpiri::InputVoltageRange::{Appliance, UPS};
    use crate::commands::qpiri::MachineType::{GridTie, Hybrid, OffGrid};
    use crate::commands::qpiri::OutputMode::{
        ParallelOutput, Phase1Of3Output, Phase2Of3Output, Phase3Of3Output, SingleMachineOutput,
    };
    use crate::commands::qpiri::OutputSourcePriority::{
        GridFirst as OutputSourceGridFirst, SBUFirst, SolarFirst as OutputSourceSolarFirst,
    };
    use crate::commands::qpiri::Topology::{Transformer, Transformerless};
    use crate::commands::qpiri::{
        BatteryType, ChargeSourcePriority, InputVoltageRange, MachineType, OutputMode,
        OutputSourcePriority, QPIRIResponse, Topology, QPIRI,
    };
    use crate::error::Result;
    use bytes::{Buf, BytesMut};
    use crc_any::CRCu16;
    use rand::{thread_rng, Rng};
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_qpiri_payload_encode() -> Result<()> {
        let req: <QPIRI as Command>::Request = ();
        assert_eq!(req.encode()?, None);
        Ok(())
    }

    #[test]
    fn test_qpiri_payload_decode_custom() -> Result<()> {
        let res = "230.0 13.0 230.0 50.0 13.0 3000 2400 24.0 23.0 21.0 28.2 27.0 0 30 60 0 0 0 - 01 1 0 27.0 0 0";

        let mut buf = BytesMut::from(res);
        let item = <QPIRI as Command>::Response::decode(&mut buf)?;
        // println!("Test result: {:#?}", item);
        assert_eq!(
            item,
            QPIRIResponse {
                grid_rating_voltage: 230.0,
                grid_rating_current: 13.0,
                ac_output_rating_voltage: 230.0,
                ac_out_rating_frequency: 50.0,
                ac_out_rating_current: 13.0,
                ac_out_rating_apparent_power: 3000,
                ac_out_rating_active_power: 2400,
                battery_rating_voltage: 24.0,
                battery_recharge_voltage: 23.0,
                battery_under_voltage: 21.0,
                battery_bulk_voltage: 28.2,
                battery_float_voltage: 27.0,
                battery_type: BatteryType::AGM,
                max_ac_charging_current: 30,
                max_charging_current: 60,
                input_voltage_range: InputVoltageRange::Appliance,
                output_source_priority: OutputSourcePriority::GridFirst,
                charge_source_priority: ChargeSourcePriority::GridFirst,
                machine_type: MachineType::OffGrid,
                topology: Topology::Transformer,
                output_mode: OutputMode::SingleMachineOutput,
                battery_redischarge_voltage: 27.0
            }
        );

        Ok(())
    }

    #[test]
    fn test_qpiri_command_encode() -> Result<()> {
        let mut codec = Codec::<QPIRI>::new();

        let mut buf = BytesMut::new();
        codec.encode((), &mut buf)?;

        assert_eq!(buf.bytes(), b"QPIRI\xf8\x54\r");

        Ok(())
    }

    #[test]
    fn test_qpiri_command_decode() -> Result<()> {
        let mut codec = Codec::<QPIRI>::new();

        for _ in 0..1000 {
            let mut rng = thread_rng();

            let grid_rating_voltage: f32 = (rng.gen_range(0.0, 500.0) * 10.0f32).floor() / 10.0;
            let grid_rating_current: f32 = (rng.gen_range(0.0, 500.0) * 10.0f32).floor() / 10.0;
            let ac_output_rating_voltage: f32 =
                (rng.gen_range(0.0, 500.0) * 10.0f32).floor() / 10.0;
            let ac_out_rating_frequency: f32 = (rng.gen_range(0.0, 60.0) * 10.0f32).floor() / 10.0;
            let ac_out_rating_current: f32 = (rng.gen_range(0.0, 500.0) * 10.0f32).floor() / 10.0;
            let ac_out_rating_apparent_power: i32 = rng.gen_range(0, 5000);
            let ac_out_rating_active_power: i32 = rng.gen_range(0, 2500);
            let battery_rating_voltage: f32 = (rng.gen_range(0.0, 60.0) * 10.0f32).floor() / 10.0;
            let battery_recharge_voltage: f32 = (rng.gen_range(0.0, 60.0) * 10.0f32).floor() / 10.0;
            let battery_under_voltage: f32 = (rng.gen_range(0.0, 60.0) * 10.0f32).floor() / 10.0;
            let battery_bulk_voltage: f32 = (rng.gen_range(0.0, 60.0) * 10.0f32).floor() / 10.0;
            let battery_float_voltage: f32 = (rng.gen_range(0.0, 60.0) * 10.0f32).floor() / 10.0;
            let max_ac_charging_current: i32 = rng.gen_range(0, 500);
            let max_charging_current: i32 = rng.gen_range(0, 500);
            let battery_redischarge_voltage: f32 =
                (rng.gen_range(0.0, 60.0) * 10.0f32).floor() / 10.0;

            let battery_type = rng.gen_range(0, 2);
            let input_voltage_range = rng.gen_range(0, 1);
            let output_source_priority = rng.gen_range(0, 2);
            let charge_source_priority = rng.gen_range(0, 3);
            let machine_type = rng.gen_range(0, 2);
            let output_mode = rng.gen_range(0, 4);
            let topology = rng.gen_range(0, 1);

            let mut res = format!(
                // 230.0  13.0    230.0   50.0    13.0    3000  2400  24.0    23.0    21.0    28.2    27.0    0     30    60    0      0    0     -  01     1     0     27.0   0 0
                "({:03.1} {:02.1} {:03.1} {:02.1} {:02.1} {:04} {:04} {:02.1} {:02.1} {:02.1} {:02.1} {:02.1} {:01} {:02} {:02} {:01} {:01} {:01} - {:02} {:01} {:01} {:02.1} 0 0",
                grid_rating_voltage,
                grid_rating_current,
                ac_output_rating_voltage,
                ac_out_rating_frequency,
                ac_out_rating_current,
                ac_out_rating_apparent_power,
                ac_out_rating_active_power,
                battery_rating_voltage,
                battery_recharge_voltage,
                battery_under_voltage,
                battery_bulk_voltage,
                battery_float_voltage,
                battery_type,
                max_ac_charging_current,
                max_charging_current,
                input_voltage_range,
                output_source_priority,
                charge_source_priority,
               match machine_type {
                        0 => "00",
                        1 => "01",
                        2 => "10",
                        _ => unreachable!(),
                    },
                topology,
                output_mode,
                battery_redischarge_voltage
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
                Some(QPIRIResponse {
                    grid_rating_voltage,
                    grid_rating_current,
                    ac_output_rating_voltage,
                    ac_out_rating_frequency,
                    ac_out_rating_current,
                    ac_out_rating_apparent_power,
                    ac_out_rating_active_power,
                    battery_rating_voltage,
                    battery_recharge_voltage,
                    battery_under_voltage,
                    battery_bulk_voltage,
                    battery_float_voltage,
                    battery_type: match battery_type {
                        0 => AGM,
                        1 => Flooded,
                        2 => User,
                        _ => unreachable!(),
                    },
                    max_ac_charging_current,
                    max_charging_current,
                    input_voltage_range: match input_voltage_range {
                        0 => Appliance,
                        1 => UPS,
                        _ => unreachable!(),
                    },
                    output_source_priority: match output_source_priority {
                        0 => OutputSourceGridFirst,
                        1 => OutputSourceSolarFirst,
                        2 => SBUFirst,
                        _ => unreachable!(),
                    },
                    charge_source_priority: match charge_source_priority {
                        0 => ChargeSourceGridFirst,
                        1 => ChargeSourceSolarFirst,
                        2 => SolarAndGrid,
                        3 => OnlySolar,
                        _ => unreachable!(),
                    },
                    machine_type: match machine_type {
                        0 => GridTie,
                        1 => OffGrid,
                        2 => Hybrid,
                        _ => unreachable!(),
                    },
                    topology: match topology {
                        0 => Transformerless,
                        1 => Transformer,
                        _ => unreachable!(),
                    },
                    output_mode: match output_mode {
                        0 => SingleMachineOutput,
                        1 => ParallelOutput,
                        2 => Phase1Of3Output,
                        3 => Phase2Of3Output,
                        4 => Phase3Of3Output,
                        _ => unreachable!(),
                    },
                    battery_redischarge_voltage
                })
            );
        }

        Ok(())
    }
}

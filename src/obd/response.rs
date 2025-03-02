use crate::vehicle::data::VehicleData;
use socketcan::{CanDataFrame, embedded_can::Frame};

pub fn parse_obd_response(frame: &CanDataFrame, data: &mut VehicleData) {
    let bytes = frame.data();
    if bytes.len() >= 4 {
        match bytes[2] {
            0x04 => {
                data.engine_load = (bytes[3] * 100) / 255;
            }
            0x05 => {
                data.coolant_temp = bytes[3] as i16 - 40;
            }
            0x06 => {
                data.fuel_trim_short_b1 = (bytes[3] as f32 - 128.0) * 100.0 / 128.0;
            }
            0x07 => {
                data.fuel_trim_long_b1 = (bytes[3] as f32 - 128.0) * 100.0 / 128.0;
            }
            0x0A => {
                data.fuel_pressure = bytes[3] * 3;
            }
            0x0B => {
                data.intake_pressure = bytes[3];
            }
            0x0C => {
                if bytes.len() >= 5 {
                    data.engine_rpm = (((bytes[3] as u16) << 8 | bytes[4] as u16) as f32) / 4.0;
                }
            }
            0x0D => {
                data.vehicle_speed = bytes[3];
            }
            0x0E => {
                data.timing_advance = (bytes[3] as f32 / 2.0) - 64.0;
            }
            0x0F => {
                data.intake_temp = bytes[3] as i16 - 40;
            }
            0x10 => {
                if bytes.len() >= 5 {
                    data.maf_sensor = (((bytes[3] as u16) << 8 | bytes[4] as u16) as f32) / 100.0;
                }
            }
            0x11 => {
                data.throttle_pos = (bytes[3] * 100) / 255;
            }
            0x14 => {
                data.o2_voltage = bytes[3] as f32 / 200.0;
            }
            0x23 => {
                if bytes.len() >= 5 {
                    data.fuel_rail_pressure = ((bytes[3] as u16) << 8 | bytes[4] as u16) * 10;
                }
            }
            0x2C => {
                data.commanded_egr = (bytes[3] * 100) / 255;
            }
            0x2D => {
                data.egr_error = (bytes[3] as f32 - 128.0) * 100.0 / 128.0;
            }
            0x2F => {
                data.fuel_level = (bytes[3] * 100) / 255;
            }
            0x33 => {
                data.baro_pressure = bytes[3];
            }
            0x42 => {
                if bytes.len() >= 5 {
                    data.control_module_voltage =
                        ((bytes[3] as u16) << 8 | bytes[4] as u16) as f32 / 1000.0;
                }
            }
            0x44 => {
                if bytes.len() >= 5 {
                    data.command_equiv_ratio =
                        ((bytes[3] as u16) << 8 | bytes[4] as u16) as f32 / 32768.0;
                }
            }
            0x46 => {
                data.ambient_temp = bytes[3] as i16 - 40;
            }
            _ => {}
        }
    }
}

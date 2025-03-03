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
            0x08 => {
                data.fuel_trim_short_b2 = (bytes[3] as f32 - 128.0) * 100.0 / 128.0;
            }
            0x09 => {
                data.fuel_trim_long_b2 = (bytes[3] as f32 - 128.0) * 100.0 / 128.0;
            }
            0x0A => {
                data.fuel_pressure = bytes[3] * 3; // kPa
            }
            0x0B => {
                data.intake_pressure = bytes[3]; // kPa
            }
            0x0C => {
                if bytes.len() >= 5 {
                    data.engine_rpm = ((bytes[3] as u16) << 8 | bytes[4] as u16) as f32 / 4.0;
                }
            }
            0x0D => {
                data.vehicle_speed = bytes[3]; // km/h
            }
            0x0E => {
                data.timing_advance = (bytes[3] as f32 / 2.0) - 64.0; // degrees before TDC
            }
            0x0F => {
                data.intake_temp = bytes[3] as i16 - 40; // °C
            }
            0x10 => {
                if bytes.len() >= 5 {
                    data.maf_sensor = ((bytes[3] as u16) << 8 | bytes[4] as u16) as f32 / 100.0; // g/s
                }
            }
            0x11 => {
                data.throttle_pos = (bytes[3] * 100) / 255; // %
            }
            0x14..=0x17 => {
                // O2 Sensors 1-4 Bank 1
                match bytes[2] {
                    0x14 => data.o2_sensor_voltage_b1s1 = bytes[3] as f32 / 200.0, // V
                    0x15 => data.o2_sensor_voltage_b1s2 = bytes[3] as f32 / 200.0, // V
                    0x16 => data.o2_sensor_voltage_b1s3 = bytes[3] as f32 / 200.0, // V
                    0x17 => data.o2_sensor_voltage_b1s4 = bytes[3] as f32 / 200.0, // V
                    _ => {}
                }
            }
            0x1F => {
                if bytes.len() >= 5 {
                    data.engine_run_time = ((bytes[3] as u16) << 8 | bytes[4] as u16) as f32;
                }
            }
            0x21 => {
                if bytes.len() >= 5 {
                    data.distance_with_mil = (bytes[3] as u16) << 8 | bytes[4] as u16; // km
                }
            }
            0x22 => {
                if bytes.len() >= 5 {
                    data.fuel_rail_pressure = ((bytes[3] as u16) << 8 | bytes[4] as u16) as u16;
                }
            }
            0x23 => {
                if bytes.len() >= 5 {
                    data.fuel_rail_pressure = ((bytes[3] as u16) << 8 | bytes[4] as u16) * 10; // kPa
                }
            }
            0x2C => {
                data.commanded_egr = (bytes[3] * 100) / 255; // %
            }
            0x2D => {
                data.egr_error = (bytes[3] as f32 - 128.0) * 100.0 / 128.0; // %
            }
            0x2E => {
                data.commanded_evap_purge = (bytes[3] * 100) / 255; // %
            }
            0x2F => {
                data.fuel_level = (bytes[3] * 100) / 255; // %
            }
            0x30 => {
                data.warmups_since_codes_cleared = bytes[3];
            }
            0x31 => {
                if bytes.len() >= 5 {
                    data.distance_since_codes_cleared = (bytes[3] as u16) << 8 | bytes[4] as u16; // km
                }
            }
            0x33 => {
                data.baro_pressure = bytes[3]; // kPa
            }
            0x42 => {
                if bytes.len() >= 5 {
                    data.control_module_voltage =
                        ((bytes[3] as u16) << 8 | bytes[4] as u16) as f32 / 1000.0; // V
                }
            }
            0x43 => {
                if bytes.len() >= 5 {
                    data.absolute_load =
                        ((bytes[3] as u16) << 8 | bytes[4] as u16) as f32 * 100.0 / 255.0; // %
                }
            }
            0x44 => {
                if bytes.len() >= 5 {
                    data.command_equiv_ratio =
                        ((bytes[3] as u16) << 8 | bytes[4] as u16) as f32 / 32768.0;
                }
            }
            0x45 => {
                data.relative_throttle_pos = (bytes[3] * 100) / 255; // %
            }
            0x46 => {
                data.ambient_temp = bytes[3] as i16 - 40; // °C
            }
            0x47 => {
                data.absolute_throttle_pos_b = (bytes[3] * 100) / 255; // %
            }
            0x48 => {
                data.absolute_throttle_pos_c = (bytes[3] * 100) / 255; // %
            }
            0x49 => {
                data.accelerator_pedal_pos_d = (bytes[3] * 100) / 255; // %
            }
            0x4A => {
                data.accelerator_pedal_pos_e = (bytes[3] * 100) / 255; // %
            }
            0x4B => {
                data.accelerator_pedal_pos_f = (bytes[3] * 100) / 255; // %
            }
            0x4C => {
                data.commanded_throttle_actuator = (bytes[3] * 100) / 255; // %
            }
            0x4D => {
                if bytes.len() >= 5 {
                    data.time_with_mil = (bytes[3] as u16) << 8 | bytes[4] as u16; // minutes
                }
            }
            0x4E => {
                if bytes.len() >= 5 {
                    data.time_since_codes_cleared = (bytes[3] as u16) << 8 | bytes[4] as u16; // minutes
                }
            }
            0x52 => {
                data.ethanol_fuel = (bytes[3] * 100) / 255; // %
            }
            0x5C => {
                data.engine_oil_temp = bytes[3] as i16 - 40; // °C
            }
            0x5E => {
                if bytes.len() >= 5 {
                    data.engine_fuel_rate =
                        ((bytes[3] as u16) << 8 | bytes[4] as u16) as f32 / 20.0; // L/h
                }
            }
            0x69 => {
                if bytes.len() >= 5 {
                    data.actual_egr = bytes[3];
                }
            }
            0x6B => {
                if bytes.len() >= 5 {
                    data.egr_temp = ((bytes[3] as u16) << 8 | bytes[4] as u16) as i16 - 40;
                }
            }
            0x73 => {
                if bytes.len() >= 5 {
                    data.exhaust_pressure = (bytes[3] as u16) << 8 | bytes[4] as u16;
                }
            }
            0x74 => {
                if bytes.len() >= 5 {
                    data.turbo_rpm = ((bytes[3] as u32) << 8 | bytes[4] as u32) * 10;
                }
            }
            0x75 | 0x76 => {
                if bytes.len() >= 5 {
                    let temp = ((bytes[3] as i16) << 8 | bytes[4] as i16) - 40;
                    if bytes[2] == 0x75 {
                        data.turbo_temp_1 = temp;
                    } else {
                        data.turbo_temp_2 = temp;
                    }
                }
            }
            0x77 => {
                if bytes.len() >= 5 {
                    data.charge_air_temp = ((bytes[3] as i16) << 8 | bytes[4] as i16) - 40;
                }
            }
            0xA2 => {
                if bytes.len() >= 5 {
                    data.fuel_rate_mg = ((bytes[3] as u16) << 8 | bytes[4] as u16) as f32 / 32.0;
                }
            }
            0xA4 => {
                if bytes.len() >= 5 {
                    data.actual_gear = ((bytes[3] as u16) << 8 | bytes[4] as u16) as f32 / 1000.0;
                }
            }
            0xA5 => {
                data.def_dosing = bytes[3] as f32 / 2.0;
            }
            0xA6 => {
                if bytes.len() >= 7 {
                    data.odometer = ((bytes[3] as u32) << 24
                        | (bytes[4] as u32) << 16
                        | (bytes[5] as u32) << 8
                        | bytes[6] as u32) as f32
                        / 10.0;
                }
            }
            _ => {}
        }
    }
}

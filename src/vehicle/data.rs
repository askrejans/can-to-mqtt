#[derive(Default, Debug)]
pub struct VehicleData {
    // Engine and Performance
    pub engine_load: u8,
    pub engine_rpm: f32,
    pub vehicle_speed: u8,
    pub timing_advance: f32,
    pub throttle_pos: u8,
    pub relative_throttle_pos: u8,
    pub absolute_throttle_pos_b: u8,
    pub absolute_throttle_pos_c: u8,
    pub accelerator_pedal_pos_d: u8,
    pub accelerator_pedal_pos_e: u8,
    pub accelerator_pedal_pos_f: u8,
    pub commanded_throttle_actuator: u8,

    // Temperature Data
    pub coolant_temp: i16,
    pub intake_temp: i16,
    pub ambient_temp: i16,
    pub engine_oil_temp: i16,
    pub catalyst_temp_b1s1: f32,
    pub catalyst_temp_b2s1: f32,

    // Pressure Data
    pub fuel_pressure: u8,
    pub intake_pressure: u8,
    pub baro_pressure: u8,
    pub fuel_rail_pressure: u16,
    pub evap_system_vapor_pressure: f32,

    // Air and Fuel Data
    pub maf_sensor: f32,
    pub command_equiv_ratio: f32,
    pub fuel_level: u8,
    pub engine_fuel_rate: f32,
    pub fuel_injection_timing: f32,
    pub ethanol_fuel: u8,

    // O2 Sensor Data
    pub o2_voltage: f32,
    pub o2_current: f32,
    pub o2_lambda: f32,
    pub o2_sensor_voltage_b1s1: f32,
    pub o2_sensor_voltage_b1s2: f32,
    pub o2_sensor_voltage_b1s3: f32,
    pub o2_sensor_voltage_b1s4: f32,

    // Fuel Trim Data
    pub fuel_trim_short_b1: f32,
    pub fuel_trim_long_b1: f32,
    pub fuel_trim_short_b2: f32,
    pub fuel_trim_long_b2: f32,

    // EGR System
    pub commanded_egr: u8,
    pub egr_error: f32,
    pub commanded_evap_purge: u8,

    // Additional Engine Data
    pub control_module_voltage: f32,
    pub absolute_load: f32,
    pub engine_run_time: f32,
    pub distance_with_mil: u16,
    pub time_since_codes_cleared: u16,
    pub warmups_since_codes_cleared: u8,
    pub distance_since_codes_cleared: u16,
    pub time_with_mil: u16,
}

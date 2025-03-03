use crate::vehicle::data::VehicleData;
use prettytable::{Table, row};

pub fn display_vehicle_data(vehicle_data: &VehicleData) {
    print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top
    let mut table = Table::new();

    table.add_row(row!["Parameter", "Value", "Unit"]);

    // Engine and Performance
    table.add_row(row![
        "Engine Load",
        vehicle_data.engine_load.to_string(),
        "%"
    ]);
    table.add_row(row![
        "Engine RPM",
        format!("{:.0}", vehicle_data.engine_rpm),
        "RPM"
    ]);
    table.add_row(row![
        "Vehicle Speed",
        vehicle_data.vehicle_speed.to_string(),
        "km/h"
    ]);
    table.add_row(row![
        "Timing Advance",
        format!("{:.1}", vehicle_data.timing_advance),
        "°"
    ]);
    table.add_row(row![
        "Throttle Position",
        vehicle_data.throttle_pos.to_string(),
        "%"
    ]);
    table.add_row(row![
        "Relative Throttle Pos",
        vehicle_data.relative_throttle_pos.to_string(),
        "%"
    ]);
    table.add_row(row![
        "Absolute Throttle Pos B",
        vehicle_data.absolute_throttle_pos_b.to_string(),
        "%"
    ]);
    table.add_row(row![
        "Absolute Throttle Pos C",
        vehicle_data.absolute_throttle_pos_c.to_string(),
        "%"
    ]);
    table.add_row(row![
        "Accelerator Pedal Pos D",
        vehicle_data.accelerator_pedal_pos_d.to_string(),
        "%"
    ]);
    table.add_row(row![
        "Accelerator Pedal Pos E",
        vehicle_data.accelerator_pedal_pos_e.to_string(),
        "%"
    ]);
    table.add_row(row![
        "Accelerator Pedal Pos F",
        vehicle_data.accelerator_pedal_pos_f.to_string(),
        "%"
    ]);

    // Temperature Data
    table.add_row(row![
        "Coolant Temp",
        vehicle_data.coolant_temp.to_string(),
        "°C"
    ]);
    table.add_row(row![
        "Intake Temperature",
        vehicle_data.intake_temp.to_string(),
        "°C"
    ]);
    table.add_row(row![
        "Ambient Temperature",
        vehicle_data.ambient_temp.to_string(),
        "°C"
    ]);
    table.add_row(row![
        "Engine Oil Temp",
        vehicle_data.engine_oil_temp.to_string(),
        "°C"
    ]);
    table.add_row(row![
        "Catalyst Temp B1S1",
        format!("{:.1}", vehicle_data.catalyst_temp_b1s1),
        "°C"
    ]);
    table.add_row(row![
        "Catalyst Temp B2S1",
        format!("{:.1}", vehicle_data.catalyst_temp_b2s1),
        "°C"
    ]);

    // Pressure Data
    table.add_row(row![
        "Fuel Pressure",
        vehicle_data.fuel_pressure.to_string(),
        "kPa"
    ]);
    table.add_row(row![
        "Intake Pressure",
        vehicle_data.intake_pressure.to_string(),
        "kPa"
    ]);
    table.add_row(row![
        "Barometric Pressure",
        vehicle_data.baro_pressure.to_string(),
        "kPa"
    ]);
    table.add_row(row![
        "Fuel Rail Pressure",
        vehicle_data.fuel_rail_pressure.to_string(),
        "kPa"
    ]);
    table.add_row(row![
        "Evap System Vapor Pressure",
        format!("{:.1}", vehicle_data.evap_system_vapor_pressure),
        "Pa"
    ]);

    // Air and Fuel Data
    table.add_row(row![
        "MAF Rate",
        format!("{:.2}", vehicle_data.maf_sensor),
        "g/s"
    ]);
    table.add_row(row![
        "Commanded AFR",
        format!("{:.3}", vehicle_data.command_equiv_ratio),
        "λ"
    ]);
    table.add_row(row!["Fuel Level", vehicle_data.fuel_level.to_string(), "%"]);
    table.add_row(row![
        "Engine Fuel Rate",
        format!("{:.2}", vehicle_data.engine_fuel_rate),
        "L/h"
    ]);
    table.add_row(row![
        "Fuel Injection Timing",
        format!("{:.2}", vehicle_data.fuel_injection_timing),
        "°"
    ]);

    // O2 Sensor Data
    table.add_row(row![
        "O2 Sensor Voltage B1S1",
        format!("{:.3}", vehicle_data.o2_voltage),
        "V"
    ]);
    table.add_row(row![
        "O2 Sensor Current B1S1",
        format!("{:.3}", vehicle_data.o2_current),
        "mA"
    ]);
    table.add_row(row![
        "O2 Sensor Lambda B1S1",
        format!("{:.3}", vehicle_data.o2_lambda),
        "λ"
    ]);

    // Fuel Trim Data
    table.add_row(row![
        "ST Fuel Trim B1",
        format!("{:.1}", vehicle_data.fuel_trim_short_b1),
        "%"
    ]);
    table.add_row(row![
        "LT Fuel Trim B1",
        format!("{:.1}", vehicle_data.fuel_trim_long_b1),
        "%"
    ]);
    table.add_row(row![
        "ST Fuel Trim B2",
        format!("{:.1}", vehicle_data.fuel_trim_short_b2),
        "%"
    ]);
    table.add_row(row![
        "LT Fuel Trim B2",
        format!("{:.1}", vehicle_data.fuel_trim_long_b2),
        "%"
    ]);

    // EGR System
    table.add_row(row![
        "Commanded EGR",
        vehicle_data.commanded_egr.to_string(),
        "%"
    ]);
    table.add_row(row![
        "EGR Error",
        format!("{:.1}", vehicle_data.egr_error),
        "%"
    ]);
    table.add_row(row![
        "Evap Purge",
        vehicle_data.commanded_evap_purge.to_string(),
        "%"
    ]);

    // Additional Engine Data
    table.add_row(row![
        "Module Voltage",
        format!("{:.1}", vehicle_data.control_module_voltage),
        "V"
    ]);
    table.add_row(row![
        "Absolute Load Value",
        format!("{:.1}", vehicle_data.absolute_load),
        "%"
    ]);
    table.add_row(row![
        "Engine Run Time",
        format!("{:.0}", vehicle_data.engine_run_time),
        "s"
    ]);
    table.add_row(row![
        "Distance with MIL on",
        vehicle_data.distance_with_mil.to_string(),
        "km"
    ]);
    table.add_row(row![
        "Time Since Codes Cleared",
        vehicle_data.time_since_codes_cleared.to_string(),
        "min"
    ]);

    table.printstd();
}

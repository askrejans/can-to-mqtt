use crate::vehicle::data::VehicleData;
use prettytable::{Table, row};

pub fn display_vehicle_data(vehicle_data: &VehicleData) {
    print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top
    let mut table = Table::new();

    table.add_row(row!["Parameter", "Value", "Unit"]);
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
        "Coolant Temp",
        vehicle_data.coolant_temp.to_string(),
        "°C"
    ]);
    table.add_row(row![
        "Throttle Position",
        vehicle_data.throttle_pos.to_string(),
        "%"
    ]);
    table.add_row(row![
        "Intake Pressure",
        vehicle_data.intake_pressure.to_string(),
        "kPa"
    ]);
    table.add_row(row![
        "Timing Advance",
        format!("{:.1}", vehicle_data.timing_advance),
        "°"
    ]);
    table.add_row(row![
        "Intake Temperature",
        vehicle_data.intake_temp.to_string(),
        "°C"
    ]);
    table.add_row(row![
        "MAF Rate",
        format!("{:.2}", vehicle_data.maf_sensor),
        "g/s"
    ]);
    table.add_row(row![
        "Fuel Pressure",
        vehicle_data.fuel_pressure.to_string(),
        "kPa"
    ]);
    table.add_row(row![
        "O2 Sensor",
        format!("{:.3}", vehicle_data.o2_voltage),
        "V"
    ]);
    table.add_row(row!["Fuel Level", vehicle_data.fuel_level.to_string(), "%"]);
    table.add_row(row![
        "Barometric Pressure",
        vehicle_data.baro_pressure.to_string(),
        "kPa"
    ]);
    table.add_row(row![
        "Ambient Temperature",
        vehicle_data.ambient_temp.to_string(),
        "°C"
    ]);
    table.add_row(row![
        "Fuel Rail Pressure",
        vehicle_data.fuel_rail_pressure.to_string(),
        "kPa"
    ]);
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
        "Command Equiv Ratio",
        format!("{:.3}", vehicle_data.command_equiv_ratio),
        ""
    ]);
    table.add_row(row![
        "Module Voltage",
        format!("{:.1}", vehicle_data.control_module_voltage),
        "V"
    ]);

    table.printstd();
}

use crate::vehicle::data::VehicleData;
use prettytable::{Cell, Table, row};

pub fn display_vehicle_data(vehicle_data: &VehicleData) {
    // Clear screen, move cursor to top, and set small font
    print!("\x1B[2J\x1B[1;1H\x1B[?3h"); // The ?3h code enables 132 column mode

    let mut table = Table::new();

    // Helper closure to create formatted cells
    let fmt_cell = |text: &str| Cell::new(text).style_spec("b");
    let fmt_val = |val: String| Cell::new(&val);

    // Add data in groups of 4 parameters per row
    table.add_row(row![
        fmt_cell("Engine Load"),
        fmt_val(format!("{}%", vehicle_data.engine_load)),
        fmt_cell("Engine RPM"),
        fmt_val(format!("{:.0} RPM", vehicle_data.engine_rpm)),
        fmt_cell("Speed"),
        fmt_val(format!("{} km/h", vehicle_data.vehicle_speed)),
        fmt_cell("Timing Adv"),
        fmt_val(format!("{:.1}°", vehicle_data.timing_advance))
    ]);

    table.add_row(row![
        fmt_cell("Throttle"),
        fmt_val(format!("{}%", vehicle_data.throttle_pos)),
        fmt_cell("Rel Throttle"),
        fmt_val(format!("{}%", vehicle_data.relative_throttle_pos)),
        fmt_cell("Abs Thrtle B"),
        fmt_val(format!("{}%", vehicle_data.absolute_throttle_pos_b)),
        fmt_cell("Abs Thrtle C"),
        fmt_val(format!("{}%", vehicle_data.absolute_throttle_pos_c))
    ]);

    table.add_row(row![
        fmt_cell("Cool Temp"),
        fmt_val(format!("{}°C", vehicle_data.coolant_temp)),
        fmt_cell("Intake Temp"),
        fmt_val(format!("{}°C", vehicle_data.intake_temp)),
        fmt_cell("Amb Temp"),
        fmt_val(format!("{}°C", vehicle_data.ambient_temp)),
        fmt_cell("Oil Temp"),
        fmt_val(format!("{}°C", vehicle_data.engine_oil_temp))
    ]);

    table.add_row(row![
        fmt_cell("Cat B1S1"),
        fmt_val(format!("{:.1}°C", vehicle_data.catalyst_temp_b1s1)),
        fmt_cell("Cat B2S1"),
        fmt_val(format!("{:.1}°C", vehicle_data.catalyst_temp_b2s1)),
        fmt_cell("Fuel Press"),
        fmt_val(format!("{} kPa", vehicle_data.fuel_pressure)),
        fmt_cell("Intk Press"),
        fmt_val(format!("{} kPa", vehicle_data.intake_pressure))
    ]);

    table.add_row(row![
        fmt_cell("Baro Press"),
        fmt_val(format!("{} kPa", vehicle_data.baro_pressure)),
        fmt_cell("Rail Press"),
        fmt_val(format!("{} kPa", vehicle_data.fuel_rail_pressure)),
        fmt_cell("MAF Rate"),
        fmt_val(format!("{:.2} g/s", vehicle_data.maf_sensor)),
        fmt_cell("CMD AFR"),
        fmt_val(format!("{:.3} λ", vehicle_data.command_equiv_ratio))
    ]);

    table.add_row(row![
        fmt_cell("Fuel Level"),
        fmt_val(format!("{}%", vehicle_data.fuel_level)),
        fmt_cell("Fuel Rate"),
        fmt_val(format!("{:.2} L/h", vehicle_data.engine_fuel_rate)),
        fmt_cell("O2 Voltage"),
        fmt_val(format!("{:.3} V", vehicle_data.o2_voltage)),
        fmt_cell("O2 Current"),
        fmt_val(format!("{:.3} mA", vehicle_data.o2_current))
    ]);

    table.add_row(row![
        fmt_cell("ST Trim B1"),
        fmt_val(format!("{:.1}%", vehicle_data.fuel_trim_short_b1)),
        fmt_cell("LT Trim B1"),
        fmt_val(format!("{:.1}%", vehicle_data.fuel_trim_long_b1)),
        fmt_cell("ST Trim B2"),
        fmt_val(format!("{:.1}%", vehicle_data.fuel_trim_short_b2)),
        fmt_cell("LT Trim B2"),
        fmt_val(format!("{:.1}%", vehicle_data.fuel_trim_long_b2))
    ]);

    table.add_row(row![
        fmt_cell("CMD EGR"),
        fmt_val(format!("{}%", vehicle_data.commanded_egr)),
        fmt_cell("EGR Error"),
        fmt_val(format!("{:.1}%", vehicle_data.egr_error)),
        fmt_cell("Mod Volt"),
        fmt_val(format!("{:.1}V", vehicle_data.control_module_voltage)),
        fmt_cell("Abs Load"),
        fmt_val(format!("{:.1}%", vehicle_data.absolute_load))
    ]);

    table.add_row(row![
        fmt_cell("Turbo RPM"),
        fmt_val(format!("{} RPM", vehicle_data.turbo_rpm)),
        fmt_cell("Turbo Temp 1"),
        fmt_val(format!("{}°C", vehicle_data.turbo_temp_1)),
        fmt_cell("Turbo Temp 2"),
        fmt_val(format!("{}°C", vehicle_data.turbo_temp_2)),
        fmt_cell("Charge Air"),
        fmt_val(format!("{}°C", vehicle_data.charge_air_temp))
    ]);

    table.add_row(row![
        fmt_cell("DPF Press"),
        fmt_val(format!("{:.1} kPa", vehicle_data.dpf_pressure)),
        fmt_cell("DPF Temp"),
        fmt_val(format!("{}°C", vehicle_data.dpf_temp)),
        fmt_cell("Gear Ratio"),
        fmt_val(format!("{:.3}", vehicle_data.actual_gear)),
        fmt_cell("DEF Dosing"),
        fmt_val(format!("{:.1}%", vehicle_data.def_dosing))
    ]);

    table.printstd();
    print!("\x1B[?3l");
}

use can_to_mqtt::vehicle::data::VehicleData;
use futures_util::StreamExt;
use socketcan::{
    CanFrame,
    embedded_can::{Frame, Id, StandardId},
    tokio::CanSocket,
};
use std::error::Error;

use can_to_mqtt::constants::OBD_RESPONSE_ID;
use can_to_mqtt::display::display_vehicle_data;
use can_to_mqtt::obd::request::send_obd_request;
use can_to_mqtt::obd::response::parse_obd_response;

use can_to_mqtt::config::AppConfig;
use can_to_mqtt::config::load_configuration;
use can_to_mqtt::mqtt_handler::{publish_if_changed, setup_mqtt};
use paho_mqtt as mqtt;
use gumdrop::Options;
use tokio;

/// Define options for the program.
#[derive(Debug, Options)]
struct MyOptions {
    #[options(help = "print help message")]
    help: bool,

    #[options(help = "Sets a custom config file", meta = "FILE")]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let opts = MyOptions::parse_args_default_or_exit();
    let config = load_config_or_exit(opts.config.as_deref());

    let mut socket_rx =
        CanSocket::open("can0").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let socket_tx =
        CanSocket::open("can0").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut vehicle_data = VehicleData::default();

    // Separate high-frequency PIDs
    let high_freq_pids = [
        (0x0C, "Engine RPM"),
        (0x0D, "Vehicle speed"),
        (0x04, "Engine load"),
        (0x0E, "Timing advance"),
        (0x11, "Throttle position"),
    ];

    // Regular frequency PIDs
    let regular_pids = [
        // Mode 01 PIDs
        (0x05, "Coolant temperature"),
        (0x06, "Short term fuel trim Bank 1"),
        (0x07, "Long term fuel trim Bank 1"),
        (0x08, "Short term fuel trim Bank 2"),
        (0x09, "Long term fuel trim Bank 2"),
        (0x0A, "Fuel pressure"),
        (0x0B, "Intake manifold pressure"),
        (0x0F, "Intake air temperature"),
        (0x10, "MAF sensor"),
        (0x14, "O2 Sensor Voltage B1S1"),
        (0x15, "O2 Sensor Voltage B1S2"),
        (0x16, "O2 Sensor Voltage B1S3"),
        (0x17, "O2 Sensor Voltage B1S4"),
        (0x1F, "Run time since engine start"),
        (0x21, "Distance traveled with MIL on"),
        (0x22, "Fuel rail pressure relative to manifold"),
        (0x23, "Fuel rail pressure"),
        (0x2C, "Commanded EGR"),
        (0x2D, "EGR Error"),
        (0x2E, "Commanded evaporative purge"),
        (0x2F, "Fuel Level"),
        (0x30, "Warm-ups since codes cleared"),
        (0x31, "Distance traveled since codes cleared"),
        (0x33, "Barometric pressure"),
        (0x42, "Control module voltage"),
        (0x43, "Absolute load value"),
        (0x44, "Commanded equivalence ratio"),
        (0x45, "Relative throttle position"),
        (0x46, "Ambient temperature"),
        (0x47, "Absolute throttle position B"),
        (0x48, "Absolute throttle position C"),
        (0x49, "Accelerator pedal position D"),
        (0x4A, "Accelerator pedal position E"),
        (0x4B, "Accelerator pedal position F"),
        (0x4C, "Commanded throttle actuator"),
        (0x4D, "Time run with MIL on"),
        (0x4E, "Time since trouble codes cleared"),
        (0x52, "Ethanol fuel %"),
        (0x5C, "Engine oil temperature"),
        (0x5E, "Engine fuel rate"),
    ];

    let mut regular_cycle_counter = 0;

    loop {
        // Request high-frequency PIDs every cycle
        for (pid, desc) in high_freq_pids.iter() {
            if let Err(e) = send_request(&socket_tx, *pid).await {
                eprintln!("Error sending request for {}: {}", desc, e);
            }
        }

        // Request regular PIDs every 10 cycles (200ms / 20ms = 10)
        if regular_cycle_counter == 0 {
            for (pid, desc) in regular_pids.iter() {
                if let Err(e) = send_request(&socket_tx, *pid).await {
                    eprintln!("Error sending request for {}: {}", desc, e);
                }
            }
        }

        let timeout = tokio::time::sleep(tokio::time::Duration::from_millis(20));
        tokio::pin!(timeout);

        let expected_responses = if regular_cycle_counter == 0 {
            high_freq_pids.len() + regular_pids.len()
        } else {
            high_freq_pids.len()
        };

        let mut responses_received = 0;
        while responses_received < expected_responses {
            tokio::select! {
                frame = socket_rx.next() => {
                    match frame {
                        Some(Ok(frame)) => {
                            if let CanFrame::Data(frame) = frame {
                                if frame.id() == Id::Standard(StandardId::new(OBD_RESPONSE_ID).unwrap()) {
                                    parse_obd_response(&frame, &mut vehicle_data);
                                    responses_received += 1;
                                }
                            }
                        }
                        Some(Err(e)) => eprintln!("Error reading frame: {}", e),
                        None => break,
                    }
                }
                _ = &mut timeout => {
                    break;
                }
            }
        }

        display_vehicle_data(&vehicle_data);

        let mqtt_client = setup_mqtt(&config);
        if let Err(e) = publish_vehicle_data(&mqtt_client, &vehicle_data) {
            eprintln!("Error publishing to MQTT: {}", e);
        }

        // Update cycle counter
        regular_cycle_counter = (regular_cycle_counter + 1) % 10;
    }
}

/// Loads the configuration file or exits the application if an error occurs.
///
/// # Arguments
///
/// * `config_path` - An optional path to the configuration file.
///
/// # Returns
///
/// * `Config` - The loaded configuration.
fn load_config_or_exit(config_path: Option<&str>) -> AppConfig {
    match load_configuration(config_path) {
        Ok(config) => config,
        Err(err) => {
            // Handle the error gracefully, print a message, and exit
            eprintln!("Error loading configuration: {}", err);
            std::process::exit(1);
        }
    }
}

async fn send_request(socket: &CanSocket, pid: u8) -> std::io::Result<()> {
    send_obd_request(socket, pid)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub fn publish_vehicle_data(cli: &mqtt::Client, data: &VehicleData) -> Result<(), Box<dyn Error>> {
    // Engine parameters
    publish_if_changed(cli, "RPM", &data.engine_rpm.to_string(), 0)?; // RPM - matches speeduino
    publish_if_changed(cli, "TPS", &data.throttle_pos.to_string(), 0)?; // TPS - matches speeduino
    publish_if_changed(cli, "MAP", &data.intake_pressure.to_string(), 0)?; // MAP - matches speeduino
    publish_if_changed(cli, "MAT", &data.intake_temp.to_string(), 0)?; // MAT - matches speeduino
    publish_if_changed(cli, "CLT", &data.coolant_temp.to_string(), 0)?; // CLT - matches speeduino
    publish_if_changed(cli, "BAT", &data.control_module_voltage.to_string(), 0)?; // BAT - matches speeduino
    publish_if_changed(cli, "BAR", &data.baro_pressure.to_string(), 0)?; // BAR - matches speeduino
    publish_if_changed(cli, "VSS", &data.vehicle_speed.to_string(), 0)?; // VSS - matches speeduino
    publish_if_changed(cli, "GER", &data.actual_gear.to_string(), 0)?; // GER - matches speeduino

    // New codes for other parameters
    publish_if_changed(cli, "ELD", &data.engine_load.to_string(), 0)?; // Engine Load
    publish_if_changed(cli, "FST", &data.fuel_trim_short_b1.to_string(), 0)?; // Fuel Short Term B1
    publish_if_changed(cli, "FLT", &data.fuel_trim_long_b1.to_string(), 0)?; // Fuel Long Term B1
    publish_if_changed(cli, "FS2", &data.fuel_trim_short_b2.to_string(), 0)?; // Fuel Short Term B2
    publish_if_changed(cli, "FL2", &data.fuel_trim_long_b2.to_string(), 0)?; // Fuel Long Term B2
    publish_if_changed(cli, "FPR", &data.fuel_pressure.to_string(), 0)?; // FPR - matches speeduino
    publish_if_changed(cli, "TAD", &data.timing_advance.to_string(), 0)?; // TAD - matches speeduino
    publish_if_changed(cli, "MAS", &data.maf_sensor.to_string(), 0)?; // Mass Air Flow

    // O2 Sensors
    publish_if_changed(cli, "O21", &data.o2_sensor_voltage_b1s1.to_string(), 0)?;
    publish_if_changed(cli, "O22", &data.o2_sensor_voltage_b1s2.to_string(), 0)?;
    publish_if_changed(cli, "O23", &data.o2_sensor_voltage_b1s3.to_string(), 0)?;
    publish_if_changed(cli, "O24", &data.o2_sensor_voltage_b1s4.to_string(), 0)?;

    // Engine Run Data
    publish_if_changed(cli, "ERT", &data.engine_run_time.to_string(), 0)?;
    publish_if_changed(cli, "MIL", &data.distance_with_mil.to_string(), 0)?;
    publish_if_changed(cli, "FRL", &data.fuel_rail_pressure.to_string(), 0)?;

    // EGR Related
    publish_if_changed(cli, "EGR", &data.commanded_egr.to_string(), 0)?;
    publish_if_changed(cli, "EGE", &data.egr_error.to_string(), 0)?;
    publish_if_changed(cli, "EGT", &data.egr_temp.to_string(), 0)?;

    // Turbo Related
    publish_if_changed(cli, "TBR", &data.turbo_rpm.to_string(), 0)?;
    publish_if_changed(cli, "TB1", &data.turbo_temp_1.to_string(), 0)?;
    publish_if_changed(cli, "TB2", &data.turbo_temp_2.to_string(), 0)?;
    publish_if_changed(cli, "CAT", &data.charge_air_temp.to_string(), 0)?;

    // Additional Parameters
    publish_if_changed(cli, "ETH", &data.ethanol_fuel.to_string(), 0)?; // ETH - matches speeduino
    publish_if_changed(cli, "OIT", &data.engine_oil_temp.to_string(), 0)?;
    publish_if_changed(cli, "FRT", &data.engine_fuel_rate.to_string(), 0)?;
    publish_if_changed(cli, "FRM", &data.fuel_rate_mg.to_string(), 0)?; // FRM - matches speeduino
    publish_if_changed(cli, "DEF", &data.def_dosing.to_string(), 0)?;
    publish_if_changed(cli, "ODO", &data.odometer.to_string(), 0)?;

    Ok(())
}

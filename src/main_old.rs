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
use gumdrop::Options;
use paho_mqtt as mqtt;
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

    let mut socket_rx = CanSocket::open(&config.can_interface)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let socket_tx = CanSocket::open(&config.can_interface)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

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
        if let Err(e) = publish_vehicle_data(&mqtt_client, &vehicle_data, &config) {
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

pub fn publish_vehicle_data(
    cli: &mqtt::Client,
    data: &VehicleData,
    config: &AppConfig,
) -> Result<(), Box<dyn Error>> {
    let base_topic = &config.mqtt_base_topic;

    // Engine parameters
    publish_if_changed(
        cli,
        &format!("{}/RPM", base_topic),
        &data.engine_rpm.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/TPS", base_topic),
        &data.throttle_pos.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/MAP", base_topic),
        &data.intake_pressure.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/MAT", base_topic),
        &data.intake_temp.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/CLT", base_topic),
        &data.coolant_temp.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/BAT", base_topic),
        &data.control_module_voltage.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/BAR", base_topic),
        &data.baro_pressure.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/VSS", base_topic),
        &data.vehicle_speed.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/GER", base_topic),
        &data.actual_gear.to_string(),
        0,
    )?;

    // Other parameters
    publish_if_changed(
        cli,
        &format!("{}/ELD", base_topic),
        &data.engine_load.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/FST", base_topic),
        &data.fuel_trim_short_b1.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/FLT", base_topic),
        &data.fuel_trim_long_b1.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/FS2", base_topic),
        &data.fuel_trim_short_b2.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/FL2", base_topic),
        &data.fuel_trim_long_b2.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/FPR", base_topic),
        &data.fuel_pressure.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/TAD", base_topic),
        &data.timing_advance.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/MAS", base_topic),
        &data.maf_sensor.to_string(),
        0,
    )?;

    // O2 Sensors
    publish_if_changed(
        cli,
        &format!("{}/O21", base_topic),
        &data.o2_sensor_voltage_b1s1.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/O22", base_topic),
        &data.o2_sensor_voltage_b1s2.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/O23", base_topic),
        &data.o2_sensor_voltage_b1s3.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/O24", base_topic),
        &data.o2_sensor_voltage_b1s4.to_string(),
        0,
    )?;

    // Engine Run Data
    publish_if_changed(
        cli,
        &format!("{}/ERT", base_topic),
        &data.engine_run_time.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/MIL", base_topic),
        &data.distance_with_mil.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/FRL", base_topic),
        &data.fuel_rail_pressure.to_string(),
        0,
    )?;

    // EGR Related
    publish_if_changed(
        cli,
        &format!("{}/EGR", base_topic),
        &data.commanded_egr.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/EGE", base_topic),
        &data.egr_error.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/EGT", base_topic),
        &data.egr_temp.to_string(),
        0,
    )?;

    // Turbo Related
    publish_if_changed(
        cli,
        &format!("{}/TBR", base_topic),
        &data.turbo_rpm.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/TB1", base_topic),
        &data.turbo_temp_1.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/TB2", base_topic),
        &data.turbo_temp_2.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/CAT", base_topic),
        &data.charge_air_temp.to_string(),
        0,
    )?;

    // Additional Parameters
    publish_if_changed(
        cli,
        &format!("{}/ETH", base_topic),
        &data.ethanol_fuel.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/OIT", base_topic),
        &data.engine_oil_temp.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/FRT", base_topic),
        &data.engine_fuel_rate.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/FRM", base_topic),
        &data.fuel_rate_mg.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/DEF", base_topic),
        &data.def_dosing.to_string(),
        0,
    )?;
    publish_if_changed(
        cli,
        &format!("{}/ODO", base_topic),
        &data.odometer.to_string(),
        0,
    )?;

    Ok(())
}

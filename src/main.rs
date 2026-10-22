//! CAN-to-MQTT Bridge - Production-ready application
//!
//! This application reads OBD-II data from CAN bus and publishes it to MQTT broker.
//! Features include graceful shutdown, health checks, metrics, and daemon mode.

use can_to_mqtt::cli::{Cli, Commands};
use can_to_mqtt::config::{load_configuration, AppConfig};
use can_to_mqtt::constants::OBD_RESPONSE_ID;
use can_to_mqtt::display::display_vehicle_data;
use can_to_mqtt::error::{AppError, CanError, MqttError};
use can_to_mqtt::metrics::{
    record_can_error, record_can_frame_received, record_can_frame_sent, record_mqtt_publish,
    record_mqtt_publish_error, record_obd_request, record_obd_response, record_obd_timeout,
    set_can_connected, set_mqtt_connected, start_metrics_server, HealthState,
};
use can_to_mqtt::health::{clock_synced, Health};
use can_to_mqtt::mqtt_handler::{cleanup_old_values, is_connected, publish_status, setup_mqtt};
use can_to_mqtt::obd::request::send_obd_request;
use can_to_mqtt::obd::response::parse_obd_response;
use can_to_mqtt::vehicle::data::VehicleData;

use clap::Parser;
use futures_util::StreamExt;
use paho_mqtt as mqtt;
use socketcan::{
    embedded_can::{Frame, Id, StandardId},
    tokio::CanSocket,
    CanFrame,
};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize tracing early (will be reconfigured after loading config)
    init_logging(&cli.log_level);

    info!("CAN-to-MQTT Bridge starting...");

    // Handle subcommands
    match cli.effective_command() {
        Commands::ValidateConfig => {
            return validate_config_command(cli.config.as_deref());
        }
        Commands::TestConnection => {
            return test_connection_command(cli.config.as_deref()).await;
        }
        Commands::Version => {
            println!("can-to-mqtt version {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Commands::Run { .. } | Commands::Daemon { .. } => {
            // Continue with main execution
        }
    }

    // Load configuration
    let mut config = load_configuration(cli.config.as_deref())
        .map_err(|e| AppError::Config(e))?;

    // Apply CLI overrides
    if cli.no_display {
        config.no_display = true;
    }

    // Reinitialize logging with config level
    init_logging(&config.log_level);

    info!("Configuration loaded successfully");
    debug!("Config: {:?}", config);

    // Handle daemon mode
    if let Commands::Daemon { pid_file, systemd } = cli.effective_command() {
        return run_daemon_mode(&config, pid_file, *systemd).await;
    }

    // Run in normal mode
    run_application(config).await
}

/// Initialize logging/tracing
fn init_logging(level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .try_init();
}

/// Validate configuration file command
fn validate_config_command(config_path: Option<&str>) -> Result<(), AppError> {
    info!("Validating configuration...");
    
    match load_configuration(config_path) {
        Ok(config) => {
            println!("✓ Configuration is valid");
            println!("\nConfiguration details:");
            println!("  CAN Interface: {}", config.can_interface);
            println!("  MQTT Broker: {}:{}", config.mqtt_host, config.mqtt_port);
            println!("  Car: {}", config.car.as_deref().unwrap_or("(none)"));
            println!("  MQTT Topic: {}", config.mqtt_base_topic);
            println!("  MQTT QoS: {}", config.mqtt_qos);
            println!("  High-frequency polling: {}ms", config.poll_interval_high_freq_ms);
            println!("  Regular polling: {}ms", config.poll_interval_regular_ms);
            println!("  Log level: {}", config.log_level);
            println!("  Prometheus enabled: {}", config.prometheus_enabled);
            if config.prometheus_enabled {
                println!("  Prometheus port: {}", config.prometheus_port);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Configuration validation failed: {}", e);
            Err(AppError::Config(e))
        }
    }
}

/// Test CAN and MQTT connections command
async fn test_connection_command(config_path: Option<&str>) -> Result<(), AppError> {
    info!("Testing connections...");
    
    let config = load_configuration(config_path)
        .map_err(|e| AppError::Config(e))?;

    println!("Testing CAN bus connection...");
    match CanSocket::open(&config.can_interface) {
        Ok(_) => println!("✓ CAN bus connection successful ({})", config.can_interface),
        Err(e) => {
            eprintln!("✗ CAN bus connection failed: {}", e);
            return Err(AppError::Can(CanError::SocketOpen(e.to_string())));
        }
    }

    println!("\nTesting MQTT broker connection...");
    match setup_mqtt(&config) {
        Ok(client) => {
            println!("✓ MQTT broker connection successful ({}:{})", config.mqtt_host, config.mqtt_port);
            let _ = client.disconnect(None);
        }
        Err(e) => {
            eprintln!("✗ MQTT broker connection failed: {}", e);
            return Err(AppError::Mqtt(e));
        }
    }

    println!("\n✓ All connections successful!");
    Ok(())
}

/// Run in daemon mode with systemd integration
async fn run_daemon_mode(config: &AppConfig, pid_file: &str, systemd: bool) -> Result<(), AppError> {
    info!("Starting in daemon mode");
    
    // Write PID file
    std::fs::write(pid_file, std::process::id().to_string())
        .map_err(|e| AppError::Initialization(format!("Failed to write PID file: {}", e)))?;
    
    info!("PID file written: {}", pid_file);

    // Run the application
    let result = run_application(config.clone()).await;

    // Cleanup PID file
    let _ = std::fs::remove_file(pid_file);

    // Notify systemd of stopping if enabled
    if systemd {
        let _ = sd_notify::notify(false, &[sd_notify::NotifyState::Stopping]);
    }

    result
}

/// Main application logic
async fn run_application(config: AppConfig) -> Result<(), AppError> {
    // Initialize health state
    let health_state = Arc::new(HealthState::new());

    // Start metrics server if enabled
    if config.prometheus_enabled {
        info!("Starting Prometheus metrics server on port {}", config.prometheus_port);
        start_metrics_server(config.prometheus_port, health_state.clone())
            .await
            .map_err(|e| AppError::Initialization(format!("Failed to start metrics server: {}", e)))?;
    }

    // Open CAN sockets
    info!("Opening CAN interface: {}", config.can_interface);
    let mut socket_rx = CanSocket::open(&config.can_interface)
        .map_err(|e| CanError::SocketOpen(e.to_string()))?;
    let socket_tx = CanSocket::open(&config.can_interface)
        .map_err(|e| CanError::SocketOpen(e.to_string()))?;
    
    set_can_connected(true);
    *health_state.can_connected.write().await = true;
    info!("CAN interface opened successfully");

    // Setup MQTT client (once, outside loop)
    info!("Connecting to MQTT broker");
    let mqtt_client = setup_mqtt(&config).map_err(|e| AppError::Mqtt(e))?;
    set_mqtt_connected(true);
    *health_state.mqtt_connected.write().await = true;
    info!("MQTT connection established");

    // Notify systemd we're ready (if running under systemd)
    let _ = sd_notify::notify(false, &[sd_notify::NotifyState::Ready]);

    // Shared vehicle data
    let vehicle_data = Arc::new(RwLock::new(VehicleData::default()));

    // Setup graceful shutdown
    let mut shutdown_signal = setup_shutdown_signal().await;

    // Spawn display task if not disabled
    if !config.no_display && atty::is(atty::Stream::Stdout) {
        spawn_display_task(vehicle_data.clone());
    }

    // Spawn cleanup task for MQTT cache
    spawn_cleanup_task();

    // Define PIDs
    let high_freq_pids = get_high_frequency_pids();
    let regular_pids = get_regular_pids();

    let mut regular_cycle_counter = 0;
    let mut last_watchdog = SystemTime::now();
    let mut cleanup_counter = 0;

    let telemetry_boot = format!("{}-{:?}", std::process::id(), SystemTime::now());
    let mut telemetry_sequence = 0u64;
    let mut health = Health::new(&telemetry_boot);
    health.set_socket(true);
    let health_topic = format!("{}/health", config.base());
    info!("Starting main loop");

    // Main loop
    loop {
        tokio::select! {
            _ = &mut shutdown_signal => {
                info!("Shutdown signal received");
                break;
            }
            _ = async {
                // Send OBD requests
                for (pid, desc) in high_freq_pids.iter() {
                    if let Err(e) = send_request(&socket_tx, *pid).await {
                        debug!("Error sending request for {}: {}", desc, e);
                        record_can_error();
                        health.on_error("canErrors", Instant::now());
                    } else {
                        record_obd_request();
                        record_can_frame_sent();
                    }
                }

                if regular_cycle_counter == 0 {
                    for (pid, desc) in regular_pids.iter() {
                        if let Err(e) = send_request(&socket_tx, *pid).await {
                            debug!("Error sending request for {}: {}", desc, e);
                            record_can_error();
                            health.on_error("canErrors", Instant::now());
                        } else {
                            record_obd_request();
                            record_can_frame_sent();
                        }
                    }
                }

                // Wait for responses with timeout
                let timeout = tokio::time::sleep(Duration::from_millis(config.poll_interval_high_freq_ms));
                tokio::pin!(timeout);

                let expected_responses = if regular_cycle_counter == 0 {
                    high_freq_pids.len() + regular_pids.len()
                } else {
                    high_freq_pids.len()
                };

                let mut captured_packets = Vec::new();
                let mut responses_received = 0;
                let mut vehicle_data_write = vehicle_data.write().await;

                while responses_received < expected_responses {
                    tokio::select! {
                        frame = socket_rx.next() => {
                            match frame {
                                Some(Ok(frame)) => {
                                    record_can_frame_received();
                                    *health_state.last_can_frame.write().await = Some(SystemTime::now());
                                    
                                    if let CanFrame::Data(frame) = frame {
                                        if frame.id() == Id::Standard(StandardId::new(OBD_RESPONSE_ID).unwrap()) {
                                            health.on_reply(Instant::now());
                                            telemetry_sequence += 1;
                                            if let Some(packet) = can_to_mqtt::telemetry::packet(frame.data(), &telemetry_boot, telemetry_sequence, SystemTime::now()) {
                                                captured_packets.push(packet);
                                            }
                                            parse_obd_response(&frame, &mut vehicle_data_write);
                                            record_obd_response();
                                            responses_received += 1;
                                        }
                                    }
                                }
                                Some(Err(e)) => {
                                    warn!("Error reading frame: {}", e);
                                    record_can_error();
                                    health.on_error("canErrors", Instant::now());
                                }
                                None => {
                                    health.set_socket(false);
                                    break;
                                }
                            }
                        }
                        _ = &mut timeout => {
                            if responses_received < expected_responses {
                                debug!("Timeout waiting for responses, got {}/{}", responses_received, expected_responses);
                                record_obd_timeout();
                                health.on_error("timeouts", Instant::now());
                            }
                            break;
                        }
                    }
                }

                drop(vehicle_data_write);

                // Publish to MQTT
                if !is_connected(&mqtt_client) {
                    warn!("MQTT client disconnected, attempting reconnection...");
                    set_mqtt_connected(false);
                    *health_state.mqtt_connected.write().await = false;
                    
                    match can_to_mqtt::mqtt_handler::reconnect_mqtt(&mqtt_client, &config).await {
                        Ok(_) => {
                            info!("MQTT reconnection successful");
                            set_mqtt_connected(true);
                            *health_state.mqtt_connected.write().await = true;
                        }
                        Err(e) => {
                            error!("MQTT reconnection failed: {}", e);
                        }
                    }
                }

                if is_connected(&mqtt_client) {
                    for packet in captured_packets {
                        let message = paho_mqtt::Message::new(format!("{}/telemetry", config.base()), packet.to_string(), 0);
                        if let Err(error) = mqtt_client.publish(message) {
                            warn!("Canonical OBD publish failed: {}", error);
                            health.on_error("publishErrors", Instant::now());
                        } else {
                            health.on_packet(&packet, Instant::now());
                        }
                    }
                    let vehicle_data_read = vehicle_data.read().await;
                    if let Err(e) = publish_vehicle_data(&mqtt_client, &*vehicle_data_read, &config) {
                        warn!("Error publishing to MQTT: {}", e);
                        record_mqtt_publish_error();
                    } else {
                        *health_state.last_mqtt_publish.write().await = Some(SystemTime::now());
                    }
                }

                // Bridge health: every 5 s and on a state/link change.
                if is_connected(&mqtt_client) {
                    if let Some(payload) = health.due(Instant::now(), wall_ms(), clock_synced()) {
                        publish_status(&mqtt_client, &config, true);
                        let message = paho_mqtt::Message::new_retained(health_topic.as_str(), payload.to_string(), 0);
                        if let Err(e) = mqtt_client.publish(message) { warn!("Health publish failed: {}", e); }
                    }
                }

                // Update cycle counter
                regular_cycle_counter = (regular_cycle_counter + 1) % 
                    (config.poll_interval_regular_ms / config.poll_interval_high_freq_ms) as usize;

                // Periodic cleanup
                cleanup_counter += 1;
                if cleanup_counter >= 1000 { // Every 1000 cycles
                    cleanup_old_values();
                    cleanup_counter = 0;
                }

                // Systemd watchdog
                if last_watchdog.elapsed().unwrap_or_default().as_secs() >= 10 {
                    let _ = sd_notify::notify(false, &[sd_notify::NotifyState::Watchdog]);
                    last_watchdog = SystemTime::now();
                }
            } => {}
        }
    }

    // Graceful shutdown
    info!("Shutting down gracefully...");
    
    set_mqtt_connected(false);
    set_can_connected(false);

    if is_connected(&mqtt_client) {
        let payload = health.shutdown(Instant::now(), wall_ms(), clock_synced());
        let _ = mqtt_client.publish(paho_mqtt::Message::new_retained(health_topic.as_str(), payload.to_string(), 0));
        publish_status(&mqtt_client, &config, false);
    }

    if let Err(e) = mqtt_client.disconnect(None) {
        warn!("Error disconnecting MQTT client: {}", e);
    } else {
        info!("MQTT client disconnected");
    }

    info!("Shutdown complete");
    Ok(())
}

fn wall_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// Setup shutdown signal handler
async fn setup_shutdown_signal() -> tokio::sync::oneshot::Receiver<()> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    tokio::spawn(async move {
        let ctrl_c = tokio::signal::ctrl_c();
        
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate()).unwrap();
            let mut sigint = signal(SignalKind::interrupt()).unwrap();
            
            tokio::select! {
                _ = ctrl_c => {
                    info!("Received Ctrl+C");
                }
                _ = sigterm.recv() => {
                    info!("Received SIGTERM");
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT");
                }
            }
        }
        
        #[cfg(not(unix))]
        {
            let _ = ctrl_c.await;
            info!("Received Ctrl+C");
        }
        
        let _ = tx.send(());
    });
    
    rx
}

/// Spawn display update task
fn spawn_display_task(vehicle_data: Arc<RwLock<VehicleData>>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        loop {
            interval.tick().await;
            let data = vehicle_data.read().await;
            display_vehicle_data(&*data);
        }
    });
}

/// Spawn periodic cleanup task
fn spawn_cleanup_task() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
        loop {
            interval.tick().await;
            cleanup_old_values();
        }
    });
}

/// Send OBD request
async fn send_request(socket: &CanSocket, pid: u8) -> Result<(), CanError> {
    send_obd_request(socket, pid)
        .await
        .map_err(|e| CanError::FrameSend(e.to_string()))
}

/// Get high-frequency PIDs
fn get_high_frequency_pids() -> Vec<(u8, &'static str)> {
    vec![
        (0x0C, "Engine RPM"),
        (0x0D, "Vehicle speed"),
        (0x04, "Engine load"),
        (0x0E, "Timing advance"),
        (0x11, "Throttle position"),
    ]
}

/// Get regular frequency PIDs
fn get_regular_pids() -> Vec<(u8, &'static str)> {
    vec![
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
        // Additional PIDs that were parsed but not requested
        (0x69, "Actual EGR"),
        (0x6B, "EGR Temperature"),
        (0x73, "Exhaust pressure"),
        (0x74, "Turbo RPM"),
        (0x75, "Turbo temperature 1"),
        (0x76, "Turbo temperature 2"),
        (0x77, "Charge air cooler temperature"),
        (0xA2, "Fuel rate (mg/stroke)"),
        (0xA4, "Actual engine gear"),
        (0xA5, "DEF dosing"),
        (0xA6, "Odometer"),
    ]
}

/// Publish vehicle data to MQTT
fn publish_vehicle_data(
    cli: &mqtt::Client,
    data: &VehicleData,
    config: &AppConfig,
) -> Result<(), MqttError> {
    use can_to_mqtt::mqtt_handler::publish_if_changed;
    
    let base_topic = &config.mqtt_base_topic;
    let qos = config.mqtt_qos;

    // Helper macro to reduce boilerplate
    macro_rules! publish {
        ($topic:expr, $value:expr) => {
            if let Err(e) = publish_if_changed(cli, &format!("{}{}", base_topic, $topic), &$value.to_string(), qos) {
                warn!("Failed to publish {}: {}", $topic, e);
                record_mqtt_publish_error();
            } else {
                record_mqtt_publish();
            }
        };
    }

    // Engine parameters
    publish!("/RPM", data.engine_rpm);
    publish!("/TPS", data.throttle_pos);
    publish!("/MAP", data.intake_pressure);
    publish!("/MAT", data.intake_temp);
    publish!("/CLT", data.coolant_temp);
    publish!("/BAT", data.control_module_voltage);
    publish!("/BAR", data.baro_pressure);
    publish!("/VSS", data.vehicle_speed);
    publish!("/GER", data.actual_gear);
    publish!("/ELD", data.engine_load);
    publish!("/FST", data.fuel_trim_short_b1);
    publish!("/FLT", data.fuel_trim_long_b1);
    publish!("/FS2", data.fuel_trim_short_b2);
    publish!("/FL2", data.fuel_trim_long_b2);
    publish!("/FPR", data.fuel_pressure);
    publish!("/TAD", data.timing_advance);
    publish!("/MAS", data.maf_sensor);

    // O2 Sensors
    publish!("/O21", data.o2_sensor_voltage_b1s1);
    publish!("/O22", data.o2_sensor_voltage_b1s2);
    publish!("/O23", data.o2_sensor_voltage_b1s3);
    publish!("/O24", data.o2_sensor_voltage_b1s4);

    // Engine Run Data
    publish!("/ERT", data.engine_run_time);
    publish!("/MIL", data.distance_with_mil);
    publish!("/FRL", data.fuel_rail_pressure);

    // EGR Related
    publish!("/EGR", data.commanded_egr);
    publish!("/EGE", data.egr_error);
    publish!("/EGT", data.egr_temp);

    // Turbo Related
    publish!("/TBR", data.turbo_rpm);
    publish!("/TB1", data.turbo_temp_1);
    publish!("/TB2", data.turbo_temp_2);
    publish!("/CAT", data.charge_air_temp);

    // Additional Parameters
    publish!("/ETH", data.ethanol_fuel);
    publish!("/OIT", data.engine_oil_temp);
    publish!("/FRT", data.engine_fuel_rate);
    publish!("/FRM", data.fuel_rate_mg);
    publish!("/DEF", data.def_dosing);
    publish!("/ODO", data.odometer);

    Ok(())
}

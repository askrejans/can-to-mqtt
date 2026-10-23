use crate::config::AppConfig;
use crate::error::{MqttError, MqttResult};
use paho_mqtt as mqtt;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, warn};

lazy_static::lazy_static! {
    static ref LAST_VALUES: Mutex<HashMap<String, (String, SystemTime)>> = Mutex::new(HashMap::new());
}

/// Cleanup old entries from LAST_VALUES cache to prevent unbounded growth
pub fn cleanup_old_values() {
    if let Ok(mut last_values) = LAST_VALUES.lock() {
        let cutoff = SystemTime::now() - Duration::from_secs(3600); // 1 hour
        last_values.retain(|_, (_, timestamp)| *timestamp > cutoff);
        debug!(
            "Cleaned up LAST_VALUES cache, current size: {}",
            last_values.len()
        );
    }
}

/// Set up and return an MQTT client based on the provided configuration.
///
/// This function takes an `AppConfig` reference, extracts MQTT-related information
/// (host and port) from it, creates an MQTT client, sets a timeout, and attempts to connect to the broker.
///
/// # Arguments
///
/// * `config` - A reference to the `AppConfig` struct containing MQTT configuration information.
///
/// # Returns
///
/// Returns a Result with an MQTT client upon successful setup and connection,
/// or an MqttError if setup or connection fails.
pub fn setup_mqtt(config: &AppConfig) -> MqttResult<mqtt::Client> {
    // Format the MQTT broker host and port.
    let host = format!(
        "{}://{}:{}",
        if config.mqtt_tls { "ssl" } else { "mqtt" },
        config.mqtt_host,
        config.mqtt_port
    );

    info!("Creating MQTT client for broker: {}", host);

    // Create client ID (use configured one or generate)
    let client_id = config
        .mqtt_client_id
        .clone()
        .unwrap_or_else(|| format!("can-to-mqtt-{}", std::process::id()));

    // Build create options
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(&host)
        .client_id(&client_id)
        .finalize();

    // Create an MQTT client.
    let mut cli =
        mqtt::Client::new(create_opts).map_err(|e| MqttError::ClientCreation(e.to_string()))?;

    // Set a timeout for synchronous calls.
    cli.set_timeout(Duration::from_secs(5));

    // Build connection options
    let mut conn_opts_builder = mqtt::ConnectOptionsBuilder::new();
    conn_opts_builder
        .keep_alive_interval(Duration::from_secs(config.mqtt_keep_alive_secs))
        .clean_session(false)
        .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(60))
        // Availability (Home Assistant / G86 car computer): retained, LWT `offline`.
        .will_message(mqtt::Message::new_retained(
            status_topic(config),
            "offline",
            1,
        ));

    // Add authentication if configured
    if let Some(ref username) = config.mqtt_username {
        conn_opts_builder.user_name(username);
        if let Some(ref password) = config.mqtt_password {
            conn_opts_builder.password(password.as_bytes());
        }
    }

    if config.mqtt_tls {
        let mut ssl = mqtt::SslOptionsBuilder::new();
        ssl.enable_server_cert_auth(true).verify(true);
        if let Some(path) = &config.mqtt_ca_file {
            ssl.trust_store(path).map_err(|_| {
                MqttError::InvalidConfig("Could not load the broker CA certificate".into())
            })?;
        }
        conn_opts_builder.ssl_options(ssl.finalize());
    }
    let conn_opts = conn_opts_builder.finalize();

    // Attempt to connect to the MQTT broker
    info!("Connecting to MQTT broker...");
    cli.connect(conn_opts)
        .map_err(|e| MqttError::Connection(e.to_string()))?;

    info!("Successfully connected to MQTT broker");
    publish_status(&cli, config, true);

    // Return the configured and connected MQTT client.
    Ok(cli)
}

/// `<base>/status` availability topic.
pub fn status_topic(config: &AppConfig) -> String {
    format!("{}/status", config.base())
}

/// Publish `online`/`offline` (retained). Re-sent with every health message so
/// a broker that delivered the LWT during a reconnect is corrected.
pub fn publish_status(cli: &mqtt::Client, config: &AppConfig, online: bool) {
    let message = mqtt::Message::new_retained(
        status_topic(config),
        if online { "online" } else { "offline" },
        1,
    );
    if let Err(e) = cli.publish(message) {
        warn!("Status publish failed: {}", e);
    }
}

/// Attempt to reconnect to MQTT broker with exponential backoff
pub async fn reconnect_mqtt(client: &mqtt::Client, _config: &AppConfig) -> MqttResult<()> {
    let mut attempt = 0;
    let max_attempts = 10;

    while attempt < max_attempts {
        attempt += 1;
        let delay = std::cmp::min(2u64.pow(attempt), 60); // Max 60 seconds

        warn!("MQTT reconnection attempt {} of {}", attempt, max_attempts);

        // The paho-mqtt client has automatic reconnection built in,
        // so we just need to wait and check connection status
        match client.reconnect() {
            Ok(_) => {
                info!("Successfully reconnected to MQTT broker");
                return Ok(());
            }
            Err(e) => {
                error!("Reconnection attempt {} failed: {}", attempt, e);
                if attempt < max_attempts {
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                }
            }
        }
    }

    Err(MqttError::Connection(format!(
        "Failed to reconnect after {} attempts",
        max_attempts
    )))
}

/// Check if MQTT client is connected
pub fn is_connected(client: &mqtt::Client) -> bool {
    client.is_connected()
}

/// Publish an MQTT message only if the value has changed since last publication
///
/// # Arguments
///
/// * `cli` - A reference to the MQTT client
/// * `topic` - The MQTT topic to publish to
/// * `payload` - The message payload
/// * `qos` - Quality of Service level (0, 1, or 2)
///
/// # Returns
///
/// Returns `Result<(), MqttError>` indicating success or if an error occurred
pub fn publish_if_changed(
    cli: &mqtt::Client,
    topic: &str,
    payload: &str,
    qos: i32,
) -> MqttResult<()> {
    // Validate inputs
    if topic.is_empty() || payload.is_empty() {
        return Err(MqttError::InvalidConfig(
            "Empty topic or payload".to_string(),
        ));
    }

    if !(0..=2).contains(&qos) {
        return Err(MqttError::InvalidConfig("Invalid QoS level".to_string()));
    }

    // Check connection
    if !cli.is_connected() {
        return Err(MqttError::NotConnected);
    }

    // Get lock on last values
    let mut last_values = LAST_VALUES
        .lock()
        .map_err(|_| MqttError::Publish("Failed to lock cache".to_string()))?;

    // Check if value has changed
    let should_publish = last_values
        .get(topic)
        .is_none_or(|(last_value, _)| last_value != payload);

    if should_publish {
        debug!("Publishing changed value to topic: {}", topic);

        // Create and publish message
        let msg = mqtt::MessageBuilder::new()
            .topic(topic)
            .payload(payload)
            .qos(qos)
            .retained(true)
            .finalize();

        cli.publish(msg)
            .map_err(|e| MqttError::Publish(e.to_string()))?;

        // Update stored value with timestamp after successful publish
        last_values.insert(topic.to_string(), (payload.to_string(), SystemTime::now()));

        Ok(())
    } else {
        debug!("Skipping publish - value unchanged for topic: {}", topic);
        Ok(())
    }
}

/// Publish an MQTT message to the specified topic with the given payload and QoS.
///
/// # Arguments
///
/// * `cli` - A reference to the MQTT client.
/// * `topic` - The MQTT topic to which the message will be published.
/// * `payload` - The payload of the MQTT message.
/// * `qos` - The Quality of Service level for the message.
///
/// # Returns
///
/// Returns `Result<(), MqttError>` indicating success or failure.
pub fn publish_message(cli: &mqtt::Client, topic: &str, payload: &str, qos: i32) -> MqttResult<()> {
    // For backwards compatibility, this now calls publish_if_changed
    publish_if_changed(cli, topic, payload, qos)
}

/// Compatibility scalars are emitted only for the actual fresh canonical field.
/// The old all-fields default snapshot could advertise zeros for absent sensors.
pub fn publish_fresh_legacy(cli: &mqtt::Client, config: &AppConfig, packet: &serde_json::Value) {
    let now = crate::state::wall_ms();
    let Some(stamp) = packet["timestampMs"].as_u64() else {
        return;
    };
    if stamp > now.saturating_add(500) || now.saturating_sub(stamp) > 2000 {
        return;
    };
    let Some(channels) = packet["channels"].as_object() else {
        return;
    };
    for (name, suffix) in [
        ("rpm", "/RPM"),
        ("throttle", "/TPS"),
        ("manifoldKpa", "/MAP"),
        ("intakeC", "/MAT"),
        ("coolantC", "/CLT"),
        ("batteryV", "/BAT"),
        ("baroKpa", "/BAR"),
        ("ecuSpeedKmh", "/VSS"),
        ("engineLoad", "/ELD"),
        ("fuelPressureKpa", "/FPR"),
        ("ignitionDeg", "/TAD"),
        ("massAirFlowGps", "/MAS"),
        ("oilC", "/OIT"),
    ] {
        if let Some(value) = channels.get(name) {
            let topic = format!("{}{suffix}", config.mqtt_base_topic);
            let _ = publish_if_changed(cli, &topic, &value.to_string(), config.mqtt_qos);
        }
    }
}

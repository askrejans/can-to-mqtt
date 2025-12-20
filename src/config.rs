use config::{Config, File};
use crate::error::{ConfigError, ConfigResult};
use tracing::{warn, info};

/// Struct to hold the application configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// The name of the CAN interface (e.g., "can0").
    pub can_interface: String,

    /// The MQTT broker host address.
    pub mqtt_host: String,

    /// The MQTT broker port number.
    pub mqtt_port: u16,

    /// The base topic of MQTT where data is pushed
    pub mqtt_base_topic: String,

    /// Optional MQTT username for authentication
    pub mqtt_username: Option<String>,

    /// Optional MQTT password for authentication
    pub mqtt_password: Option<String>,

    /// Optional MQTT client ID (defaults to auto-generated)
    pub mqtt_client_id: Option<String>,

    /// MQTT keep-alive interval in seconds
    pub mqtt_keep_alive_secs: u64,

    /// MQTT quality of service (0, 1, or 2)
    pub mqtt_qos: i32,

    /// High-frequency polling interval in milliseconds
    pub poll_interval_high_freq_ms: u64,

    /// Regular polling interval in milliseconds
    pub poll_interval_regular_ms: u64,

    /// Logging level (trace, debug, info, warn, error)
    pub log_level: String,

    /// Enable Prometheus metrics export
    pub prometheus_enabled: bool,

    /// Prometheus metrics HTTP server port
    pub prometheus_port: u16,

    /// List of enabled PIDs (empty means all enabled)
    pub enabled_pids: Vec<u8>,

    /// Disable terminal display output
    pub no_display: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            can_interface: "can0".to_string(),
            mqtt_host: "localhost".to_string(),
            mqtt_port: 1883,
            mqtt_base_topic: "/GOLF86/ECU/".to_string(),
            mqtt_username: None,
            mqtt_password: None,
            mqtt_client_id: None,
            mqtt_keep_alive_secs: 60,
            mqtt_qos: 0,
            poll_interval_high_freq_ms: 20,
            poll_interval_regular_ms: 200,
            log_level: "info".to_string(),
            prometheus_enabled: true,
            prometheus_port: 9090,
            enabled_pids: Vec::new(),
            no_display: false,
        }
    }
}

impl AppConfig {
    /// Validate the configuration
    pub fn validate(&self) -> ConfigResult<()> {
        // Validate MQTT port
        if self.mqtt_port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "mqtt_port".to_string(),
                reason: "Port must be between 1 and 65535".to_string(),
            });
        }

        // Validate MQTT host
        if self.mqtt_host.is_empty() {
            return Err(ConfigError::MissingField("mqtt_host".to_string()));
        }

        // Validate CAN interface
        if self.can_interface.is_empty() {
            return Err(ConfigError::MissingField("can_interface".to_string()));
        }

        // Validate MQTT QoS
        if self.mqtt_qos < 0 || self.mqtt_qos > 2 {
            return Err(ConfigError::InvalidValue {
                field: "mqtt_qos".to_string(),
                reason: "QoS must be 0, 1, or 2".to_string(),
            });
        }

        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.log_level.to_lowercase().as_str()) {
            return Err(ConfigError::InvalidValue {
                field: "log_level".to_string(),
                reason: format!("Must be one of: {}", valid_levels.join(", ")),
            });
        }

        // Validate polling intervals
        if self.poll_interval_high_freq_ms == 0 {
            return Err(ConfigError::InvalidValue {
                field: "poll_interval_high_freq_ms".to_string(),
                reason: "Must be greater than 0".to_string(),
            });
        }

        if self.poll_interval_regular_ms == 0 {
            return Err(ConfigError::InvalidValue {
                field: "poll_interval_regular_ms".to_string(),
                reason: "Must be greater than 0".to_string(),
            });
        }

        // Validate Prometheus port
        if self.prometheus_enabled && self.prometheus_port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "prometheus_port".to_string(),
                reason: "Port must be between 1 and 65535".to_string(),
            });
        }

        Ok(())
    }

    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("CAN2MQTT_CAN_INTERFACE") {
            info!("Overriding can_interface from environment: {}", val);
            self.can_interface = val;
        }

        if let Ok(val) = std::env::var("CAN2MQTT_MQTT_HOST") {
            info!("Overriding mqtt_host from environment: {}", val);
            self.mqtt_host = val;
        }

        if let Ok(val) = std::env::var("CAN2MQTT_MQTT_PORT") {
            if let Ok(port) = val.parse() {
                info!("Overriding mqtt_port from environment: {}", port);
                self.mqtt_port = port;
            }
        }

        if let Ok(val) = std::env::var("CAN2MQTT_MQTT_USERNAME") {
            info!("Overriding mqtt_username from environment");
            self.mqtt_username = Some(val);
        }

        if let Ok(val) = std::env::var("CAN2MQTT_MQTT_PASSWORD") {
            info!("MQTT password provided from environment");
            self.mqtt_password = Some(val);
        }

        if let Ok(val) = std::env::var("CAN2MQTT_LOG_LEVEL") {
            info!("Overriding log_level from environment: {}", val);
            self.log_level = val;
        }
    }
}

/// Load application configuration from a TOML file.
///
/// This function reads the configuration settings from a TOML file.
///
/// # Arguments
/// - `config_path`: An optional path to the configuration file.
///
/// # Returns
/// Returns a `Result` containing either the `AppConfig` struct with the loaded configuration or an error message.
pub fn load_configuration(config_path: Option<&str>) -> ConfigResult<AppConfig> {
    let settings = if let Some(path) = config_path {
        load_from_path(path)?
    } else {
        load_default_paths()?
    };

    let mut config = AppConfig {
        can_interface: settings
            .get_string("can_interface")
            .unwrap_or_else(|_| {
                warn!("can_interface not found in config, using default: can0");
                "can0".to_string()
            }),
        mqtt_host: settings
            .get_string("mqtt_host")
            .unwrap_or_else(|_| {
                warn!("mqtt_host not found in config, using default: localhost");
                "localhost".to_string()
            }),
        mqtt_port: settings
            .get_int("mqtt_port")
            .unwrap_or_else(|_| {
                warn!("mqtt_port not found in config, using default: 1883");
                1883
            }) as u16,
        mqtt_base_topic: settings
            .get_string("mqtt_base_topic")
            .unwrap_or_else(|_| {
                warn!("mqtt_base_topic not found in config, using default: /GOLF86/ECU/");
                "/GOLF86/ECU/".to_string()
            }),
        mqtt_username: settings.get_string("mqtt_username").ok(),
        mqtt_password: settings.get_string("mqtt_password").ok(),
        mqtt_client_id: settings.get_string("mqtt_client_id").ok(),
        mqtt_keep_alive_secs: settings
            .get_int("mqtt_keep_alive_secs")
            .unwrap_or(60) as u64,
        mqtt_qos: settings.get_int("mqtt_qos").unwrap_or(0) as i32,
        poll_interval_high_freq_ms: settings
            .get_int("poll_interval_high_freq_ms")
            .unwrap_or(20) as u64,
        poll_interval_regular_ms: settings
            .get_int("poll_interval_regular_ms")
            .unwrap_or(200) as u64,
        log_level: settings
            .get_string("log_level")
            .unwrap_or_else(|_| "info".to_string()),
        prometheus_enabled: settings
            .get_bool("prometheus_enabled")
            .unwrap_or(true),
        prometheus_port: settings
            .get_int("prometheus_port")
            .unwrap_or(9090) as u16,
        enabled_pids: settings
            .get_array("enabled_pids")
            .ok()
            .map(|arr| {
                arr.into_iter()
                    .filter_map(|v| v.into_int().ok().map(|i| i as u8))
                    .collect()
            })
            .unwrap_or_default(),
        no_display: settings.get_bool("no_display").unwrap_or(false),
    };

    // Apply environment variable overrides
    config.apply_env_overrides();

    // Validate configuration
    config.validate()?;

    Ok(config)
}

/// Loads the configuration from the specified path.
///
/// This function attempts to load the configuration from the given file path.
/// If the file is successfully loaded, the configuration is returned.
/// If there is an error loading the file, an error message is returned.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path to the configuration file.
///
/// # Returns
///
/// * `Ok(Config)` - If the configuration file is successfully loaded.
/// * `Err(ConfigError)` - If there is an error loading the configuration file.
fn load_from_path(path: &str) -> ConfigResult<Config> {
    Config::builder()
        .add_source(File::with_name(path))
        .build()
        .map_err(|err| ConfigError::ParseError(err.to_string()))
}

/// Attempts to load the configuration from default paths.
///
/// This function tries to load the configuration from the following locations in order:
/// 1. A `settings.toml` file located in the same directory as the executable.
/// 2. A `can-to-mqtt.toml` file located at `/usr/etc/g86-car-telemetry/`.
/// 3. A `can-to-mqtt.toml` file located at `/etc/g86-car-telemetry/`.
///
/// If a configuration file is successfully loaded from any of these locations, it will be used.
/// If none of the files are found or successfully loaded, the default configuration will be returned.
///
/// # Returns
///
/// * `Ok(Config)` - If a configuration file is successfully loaded from any of the default paths.
/// * `Err(ConfigError)` - If there is an error loading the configuration from all default paths.
fn load_default_paths() -> ConfigResult<Config> {
    if let Ok(exe_dir) = std::env::current_exe() {
        if let Some(exe_dir) = exe_dir.parent() {
            let default_path = exe_dir.join("settings.toml");

            if let Ok(path_str) = default_path.to_str().ok_or("Invalid path") {
                if let Ok(config) = Config::builder()
                    .add_source(File::with_name(path_str))
                    .build()
                {
                    info!("Loaded configuration from: {}", default_path.display());
                    return Ok(config);
                }
            }
        }
    }

    if let Ok(config) = Config::builder()
        .add_source(File::with_name(
            "/usr/etc/g86-car-telemetry/can-to-mqtt.toml",
        ))
        .build()
    {
        info!("Loaded configuration from: /usr/etc/g86-car-telemetry/can-to-mqtt.toml");
        return Ok(config);
    }

    if let Ok(config) = Config::builder()
        .add_source(File::with_name("/etc/g86-car-telemetry/can-to-mqtt.toml"))
        .build()
    {
        info!("Loaded configuration from: /etc/g86-car-telemetry/can-to-mqtt.toml");
        return Ok(config);
    }

    warn!("No configuration file found, using defaults");
    Ok(Config::default())
}

use config::{Config, File};
use std::path::Path;

/// Struct to hold the application configuration.
pub struct AppConfig {
    /// The name of the serial port.
    pub can_interface: String,

    /// The MQTT broker host address.
    pub mqtt_host: String,

    /// The MQTT broker port number.
    pub mqtt_port: i64,

    // The base topic of MQTT where data is pushed
    pub mqtt_base_topic: String,
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
pub fn load_configuration(config_path: Option<&str>) -> Result<AppConfig, String> {
    let settings = if let Some(path) = config_path {
        load_from_path(path)?
    } else {
        load_default_paths()?
    };

    Ok(AppConfig {
        can_interface: settings
            .get_string("can_interface")
            .unwrap_or_else(|_| "can0".to_string()),
        mqtt_host: settings
            .get_string("mqtt_host")
            .unwrap_or_else(|_| "default_host".to_string()),
        mqtt_port: settings.get_int("mqtt_port").unwrap_or(1883),
        mqtt_base_topic: settings
            .get_string("mqtt_base_topic")
            .unwrap_or_else(|_| "default_topic".to_string()),
    })
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
/// * `Err(String)` - If there is an error loading the configuration file.
fn load_from_path(path: &str) -> Result<Config, String> {
    Config::builder()
        .add_source(File::with_name(path))
        .build()
        .map_err(|err| format!("{}", err))
}

/// Attempts to load the configuration from default paths.
///
/// This function tries to load the configuration from the following locations in order:
/// 1. A `settings.toml` file located in the same directory as the executable.
/// 2. A `gps-to-mqtt.toml` file located at `/usr/etc/g86-car-telemetry/`.
/// 3. A `gps-to-mqtt.toml` file located at `/etc/g86-car-telemetry/`.
///
/// If a configuration file is successfully loaded from any of these locations, it will be used.
/// If none of the files are found or successfully loaded, the default configuration will be returned.
///
/// # Returns
///
/// * `Ok(Config)` - If a configuration file is successfully loaded from any of the default paths.
/// * `Err(String)` - If there is an error loading the configuration from all default paths.
fn load_default_paths() -> Result<Config, String> {
    if let Ok(exe_dir) = std::env::current_exe() {
        let exe_dir = exe_dir.parent().unwrap_or_else(|| Path::new("."));
        let default_path = exe_dir.join("settings.toml");

        if let Ok(config) = Config::builder()
            .add_source(File::with_name(default_path.to_str().unwrap()))
            .build()
        {
            return Ok(config);
        }
    }

    if let Ok(config) = Config::builder()
        .add_source(File::with_name(
            "/usr/etc/g86-car-telemetry/can-to-mqtt.toml",
        ))
        .build()
    {
        return Ok(config);
    }

    if let Ok(config) = Config::builder()
        .add_source(File::with_name("/etc/g86-car-telemetry/can-to-mqtt.toml"))
        .build()
    {
        return Ok(config);
    }

    Ok(Config::default())
}

use crate::error::{ConfigError, ConfigResult};
use crate::passive::PassiveSignal;
use config::{Config, File};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub const DEFAULT_BASE_TOPIC: &str = "/telemetry/can";
pub fn valid_car(car: &str) -> bool {
    !car.is_empty()
        && car.len() <= 32
        && !car.starts_with('-')
        && car
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub can_interface: String,
    pub mode: String,
    pub can_format: String,
    pub obd_response_id: Option<u32>,
    pub passive_profile: String,
    pub passive_signals: Vec<PassiveSignal>,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_base_topic: String,
    pub car: Option<String>,
    #[serde(skip)]
    pub mqtt_base_topic_explicit: bool,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
    pub mqtt_password_file: Option<String>,
    pub mqtt_client_id: Option<String>,
    pub mqtt_tls: bool,
    pub mqtt_ca_file: Option<String>,
    pub mqtt_keep_alive_secs: u64,
    pub mqtt_qos: i32,
    pub legacy_scalar_output: bool,
    pub poll_interval_high_freq_ms: u64,
    pub poll_interval_regular_ms: u64,
    pub log_level: String,
    pub prometheus_enabled: bool,
    pub prometheus_port: u16,
    pub enabled_pids: Vec<u8>,
    pub no_display: bool,
    pub http_enabled: bool,
    pub http_bind: String,
    pub http_port: u16,
    pub http_token: Option<String>,
    pub http_token_file: String,
    pub config_draft_path: String,
}
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            can_interface: "can0".into(),
            mode: "obd".into(),
            can_format: "auto".into(),
            obd_response_id: None,
            passive_profile: "can".into(),
            passive_signals: vec![],
            mqtt_host: "localhost".into(),
            mqtt_port: 1883,
            mqtt_base_topic: DEFAULT_BASE_TOPIC.into(),
            car: None,
            mqtt_base_topic_explicit: false,
            mqtt_username: None,
            mqtt_password: None,
            mqtt_password_file: None,
            mqtt_client_id: None,
            mqtt_tls: false,
            mqtt_ca_file: None,
            mqtt_keep_alive_secs: 60,
            mqtt_qos: 0,
            legacy_scalar_output: true,
            poll_interval_high_freq_ms: 20,
            poll_interval_regular_ms: 200,
            log_level: "info".into(),
            prometheus_enabled: true,
            prometheus_port: 9090,
            enabled_pids: vec![],
            no_display: false,
            http_enabled: true,
            http_bind: "127.0.0.1".into(),
            http_port: 9093,
            http_token: None,
            http_token_file: private_dir()
                .join("http-9093.token")
                .to_string_lossy()
                .into(),
            config_draft_path: private_dir()
                .join("draft-9093.json")
                .to_string_lossy()
                .into(),
        }
    }
}
impl std::fmt::Debug for AppConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.redacted().fmt(f)
    }
}
impl AppConfig {
    pub fn redacted(&self) -> Value {
        let mut value = serde_json::to_value(self).unwrap_or_else(|_| json!({}));
        if let Some(obj) = value.as_object_mut() {
            obj.insert(
                "mqtt_base_topic_explicit".into(),
                json!(self.mqtt_base_topic_explicit),
            );
            for key in ["mqtt_password", "http_token"] {
                obj.insert(
                    key.into(),
                    json!(if obj.get(key).is_some_and(|v| !v.is_null()) {
                        Some("<redacted>")
                    } else {
                        None
                    }),
                );
            }
        };
        value
    }
    pub fn validate(&self) -> ConfigResult<()> {
        let bad = |field: &str, reason: &str| ConfigError::InvalidValue {
            field: field.into(),
            reason: reason.into(),
        };
        if self.mqtt_port == 0
            || self.mqtt_host.is_empty()
            || self
                .mqtt_host
                .chars()
                .any(|c| c.is_control() || "/@?#".contains(c))
        {
            return Err(bad(
                "mqtt_host",
                "Use a broker hostname or IP and a valid port, without credentials or URL paths",
            ));
        }
        if self.can_interface.is_empty()
            || self.can_interface.len() > 15
            || !self
                .can_interface
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err(bad("can_interface", "Choose a Linux CAN interface name"));
        }
        if let Some(car) = &self.car
            && !valid_car(car)
        {
            return Err(bad("car", "Use 1–32 lowercase letters, digits or hyphens"));
        }
        if !["obd", "passive"].contains(&self.mode.as_str())
            || !["auto", "standard", "extended"].contains(&self.can_format.as_str())
        {
            return Err(bad(
                "mode",
                "Choose OBD polling or passive capture and a valid CAN format",
            ));
        }
        if self.obd_response_id.is_some_and(|id| id > 0x1FFF_FFFF) {
            return Err(bad("obd_response_id", "CAN ID exceeds 29 bits"));
        }
        if (!valid_car(&self.passive_profile) && self.passive_profile != "obd_mode01")
            || self.passive_signals.len() > 64
        {
            return Err(bad(
                "passive_profile",
                "Choose a short profile ID and at most 64 signals",
            ));
        }
        if self.mode == "passive"
            && self.passive_profile == "obd_mode01"
            && self.obd_response_id.is_none()
        {
            return Err(bad(
                "obd_response_id",
                "Bind one observed ECU response ID for passive OBD observation",
            ));
        }
        let mut names = std::collections::HashSet::new();
        for signal in &self.passive_signals {
            if !names.insert(&signal.channel) {
                return Err(bad(
                    "passive_signals",
                    "Each profile must define a channel only once",
                ));
            }
            signal.validate().map_err(|e| bad("passive_signals", &e))?
        }
        if self.mqtt_base_topic.trim_matches('/').is_empty()
            || self.mqtt_base_topic.len() > 512
            || self
                .mqtt_base_topic
                .chars()
                .any(|c| c.is_control() || "+#".contains(c))
        {
            return Err(bad(
                "mqtt_base_topic",
                "Use a nonempty topic prefix without wildcards",
            ));
        }
        if !(0..=2).contains(&self.mqtt_qos)
            || self.poll_interval_high_freq_ms < 10
            || self.poll_interval_regular_ms < self.poll_interval_high_freq_ms
            || self.poll_interval_regular_ms > 60_000
        {
            return Err(bad(
                "poll_interval_regular_ms",
                "Use polling intervals of at least 10 ms; regular must be between high-frequency and 60 seconds",
            ));
        }
        if !["trace", "debug", "info", "warn", "error"]
            .contains(&self.log_level.to_lowercase().as_str())
        {
            return Err(bad("log_level", "Choose trace, debug, info, warn or error"));
        }
        if self.prometheus_enabled && self.prometheus_port == 0 {
            return Err(bad("prometheus_port", "Choose a nonzero metrics port"));
        }
        if self.http_enabled
            && (self.http_port == 0
                || !self
                    .http_bind
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback()))
        {
            return Err(bad(
                "http_bind",
                "The native HTTP UI binds to loopback only. Use an authenticated HTTPS reverse proxy for LAN access",
            ));
        }
        if self
            .http_token
            .as_ref()
            .is_some_and(|v| v.len() < 32 || v.len() > 256 || v.chars().any(char::is_control))
        {
            return Err(bad(
                "http_token",
                "Use a private token of 32–256 printable characters",
            ));
        }
        for v in [
            self.http_token_file.as_str(),
            self.config_draft_path.as_str(),
        ] {
            if v.is_empty() || v.chars().any(char::is_control) {
                return Err(bad("private_paths", "Choose local token and draft files"));
            }
        }
        Ok(())
    }
    pub fn apply_env_overrides(&mut self) {
        for (name, target) in [
            ("CAN_INTERFACE", &mut self.can_interface),
            ("MQTT_HOST", &mut self.mqtt_host),
            ("MODE", &mut self.mode),
            ("CAN_FORMAT", &mut self.can_format),
            ("HTTP_BIND", &mut self.http_bind),
            ("LOG_LEVEL", &mut self.log_level),
        ] {
            if let Ok(v) = std::env::var(format!("CAN2MQTT_{name}")) {
                *target = v;
            }
        }
        for (name, target) in [
            ("MQTT_USERNAME", &mut self.mqtt_username),
            ("MQTT_PASSWORD", &mut self.mqtt_password),
            ("MQTT_PASSWORD_FILE", &mut self.mqtt_password_file),
            ("HTTP_TOKEN", &mut self.http_token),
        ] {
            if let Ok(v) = std::env::var(format!("CAN2MQTT_{name}")) {
                *target = Some(v)
            }
        }
        if let Ok(v) = std::env::var("CAN2MQTT_MQTT_PORT")
            && let Ok(p) = v.parse()
        {
            self.mqtt_port = p
        }
        if let Ok(v) = std::env::var("CAN2MQTT_HTTP_PORT")
            && let Ok(p) = v.parse()
        {
            self.http_port = p
        }
        if let Ok(v) = std::env::var("CAN2MQTT_MQTT_BASE_TOPIC") {
            self.mqtt_base_topic = v;
            self.mqtt_base_topic_explicit = true
        }
        if let Ok(v) = std::env::var("CAN2MQTT_CAR") {
            self.car = Some(v)
        }
    }
    pub fn resolve_base_topic(&mut self) {
        if !self.mqtt_base_topic_explicit {
            self.mqtt_base_topic = if let Some(car) = &self.car {
                if self.mode == "passive" {
                    format!("/g86/{car}/sensors/{}", self.passive_profile)
                } else {
                    format!("/g86/{car}/obd")
                }
            } else {
                DEFAULT_BASE_TOPIC.into()
            }
        }
    }
    pub fn base(&self) -> &str {
        self.mqtt_base_topic.trim_end_matches('/')
    }
}
pub fn load_configuration(path: Option<&str>) -> ConfigResult<AppConfig> {
    let selected = path
        .map(PathBuf::from)
        .or_else(|| default_paths().into_iter().find(|p| p.is_file()));
    let settings = if let Some(path) = &selected {
        info!(path=%path.display(),"Loading CAN configuration");
        Config::builder()
            .add_source(File::from(path.as_path()))
            .build()?
    } else {
        warn!("No configuration file found; using defaults");
        Config::default()
    };
    let explicit = settings.get_string("mqtt_base_topic").is_ok();
    let mut cfg: AppConfig = settings.clone().try_deserialize()?;
    cfg.mqtt_base_topic_explicit = explicit;
    cfg.apply_env_overrides();
    cfg.resolve_base_topic();
    if let Some(path) = &cfg.mqtt_password_file {
        use std::io::Read;
        let mut bytes = Vec::new();
        let file = std::fs::File::open(path).map_err(|_| ConfigError::InvalidValue {
            field: "mqtt_password_file".into(),
            reason: "Could not read the private broker password file".into(),
        })?;
        file.take(4097).read_to_end(&mut bytes).map_err(|_| {
            ConfigError::ValidationFailed("Could not read the private broker password file".into())
        })?;
        if bytes.len() > 4096 {
            return Err(ConfigError::ValidationFailed(
                "Broker password file is too large".into(),
            ));
        };
        cfg.mqtt_password = Some(
            String::from_utf8(bytes)
                .map_err(|_| ConfigError::ValidationFailed("Broker password is not UTF-8".into()))?
                .trim_end_matches(['\r', '\n'])
                .into(),
        )
    }
    if settings.get_string("http_token_file").is_err() {
        cfg.http_token_file = private_dir()
            .join(format!("http-{}.token", cfg.http_port))
            .to_string_lossy()
            .into();
    }
    if settings.get_string("config_draft_path").is_err() {
        cfg.config_draft_path = private_dir()
            .join(format!("draft-{}.json", cfg.http_port))
            .to_string_lossy()
            .into();
    }
    cfg.validate()?;
    Ok(cfg)
}
fn default_paths() -> Vec<PathBuf> {
    let mut paths = vec![Path::new("settings.toml").into()];
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        paths.push(parent.join("settings.toml"))
    }
    paths.extend([
        "/etc/can-to-mqtt/settings.toml".into(),
        "/etc/can-to-mqtt/can-to-mqtt.toml".into(),
        "/usr/etc/g86-car-telemetry/can-to-mqtt.toml".into(),
        "/etc/g86-car-telemetry/can-to-mqtt.toml".into(),
    ]);
    paths
}

fn private_dir() -> PathBuf {
    let current = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join(".config")
        })
        .join("can-to-mqtt");
    let legacy = current.with_file_name("g86-can-to-mqtt");
    if !current.exists() && legacy.exists() {
        legacy
    } else {
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn load(text: &str) -> ConfigResult<AppConfig> {
        let mut random = [0u8; 8];
        getrandom::fill(&mut random).unwrap();
        let dir = std::env::temp_dir().join(format!("g86-can-{:x}", u64::from_ne_bytes(random)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.toml");
        std::fs::write(&path, text).unwrap();
        let result = load_configuration(path.to_str());
        std::fs::remove_dir_all(dir).unwrap();
        result
    }
    #[test]
    fn preserves_legacy_and_explicit_topic_precedence() {
        assert_eq!(
            load("can_interface='can0'").unwrap().mqtt_base_topic,
            DEFAULT_BASE_TOPIC
        );
        assert_eq!(
            load("mqtt_base_topic='/GOLF86/ECU/'").unwrap().base(),
            "/GOLF86/ECU"
        );
        let paths = default_paths();
        assert!(
            paths
                .iter()
                .position(|p| p == Path::new("/etc/can-to-mqtt/settings.toml"))
                .unwrap()
                < paths
                    .iter()
                    .position(|p| p == Path::new("/etc/g86-car-telemetry/can-to-mqtt.toml"))
                    .unwrap()
        );
        assert_eq!(load("car='qa-car'").unwrap().base(), "/g86/qa-car/obd");
        assert_eq!(
            load("car='qa-car'\nmqtt_base_topic='/legacy/'")
                .unwrap()
                .base(),
            "/legacy"
        );
        assert_eq!(
            load("car='qa-car'\nmode='passive'").unwrap().base(),
            "/g86/qa-car/sensors/can"
        );
    }
    #[test]
    fn validates_privacy_and_typed_bounds_before_starting() {
        for text in [
            "car='Bad Name'",
            "mqtt_port=70000",
            "mqtt_port=-1",
            "can_interface='can0; bad'",
            "poll_interval_regular_ms=0",
            "poll_interval_regular_ms=10\npoll_interval_high_freq_ms=20",
            "http_bind='0.0.0.0'",
            "mode='passive'\npassive_profile='obd_mode01'",
            "mqtt_host='user:secret@host'",
        ] {
            assert!(load(text).is_err(), "{text}")
        }
    }
    #[test]
    fn secrets_are_redacted_even_in_debug() {
        let c = AppConfig {
            mqtt_password: Some("test-password-never-in-debug".into()),
            http_token: Some("test-token-never-in-debug-aaaaaaaaaaaa".into()),
            ..AppConfig::default()
        };
        let debug = format!("{c:?}");
        assert!(!debug.contains("test-password"));
        assert!(!debug.contains("test-token"));
        assert_eq!(c.redacted()["mqtt_password"], "<redacted>");
    }
}

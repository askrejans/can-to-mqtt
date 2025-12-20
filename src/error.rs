//! Comprehensive error types for the CAN-to-MQTT application.
//!
//! This module defines all error types used throughout the application,
//! with proper error chaining and context preservation.

use std::fmt;
use thiserror::Error;

/// Errors related to CAN bus operations
#[derive(Error, Debug)]
pub enum CanError {
    #[error("Failed to open CAN socket: {0}")]
    SocketOpen(String),

    #[error("Failed to send CAN frame: {0}")]
    FrameSend(String),

    #[error("Failed to receive CAN frame: {0}")]
    FrameReceive(String),

    #[error("Invalid CAN frame format: {0}")]
    InvalidFrame(String),

    #[error("CAN socket IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors related to MQTT operations
#[derive(Error, Debug)]
pub enum MqttError {
    #[error("Failed to create MQTT client: {0}")]
    ClientCreation(String),

    #[error("Failed to connect to MQTT broker: {0}")]
    Connection(String),

    #[error("Failed to publish MQTT message: {0}")]
    Publish(String),

    #[error("Failed to disconnect from MQTT broker: {0}")]
    Disconnect(String),

    #[error("MQTT client not connected")]
    NotConnected,

    #[error("Invalid MQTT configuration: {0}")]
    InvalidConfig(String),

    #[error("MQTT connection lost, reconnecting...")]
    ConnectionLost,
}

impl From<paho_mqtt::Error> for MqttError {
    fn from(err: paho_mqtt::Error) -> Self {
        MqttError::Connection(err.to_string())
    }
}

/// Errors related to configuration loading and validation
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Missing required configuration field: {0}")]
    MissingField(String),

    #[error("Invalid configuration value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),

    #[error("Environment variable error: {0}")]
    EnvVarError(String),
}

impl From<config::ConfigError> for ConfigError {
    fn from(err: config::ConfigError) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}

/// Errors related to OBD-II protocol operations
#[derive(Error, Debug)]
pub enum ObdError {
    #[error("Invalid OBD-II PID: 0x{0:02X}")]
    InvalidPid(u8),

    #[error("OBD-II request timeout for PID 0x{0:02X}")]
    Timeout(u8),

    #[error("Invalid OBD-II response format: {0}")]
    InvalidResponse(String),

    #[error("OBD-II response parse error for PID 0x{pid:02X}: {reason}")]
    ParseError { pid: u8, reason: String },

    #[error("Unsupported OBD-II mode: {0}")]
    UnsupportedMode(u8),

    #[error("Multi-frame response error: {0}")]
    MultiFrameError(String),

    #[error("DTC decode error: {0}")]
    DtcDecodeError(String),

    #[error("Vehicle info request failed: {0}")]
    VehicleInfoError(String),
}

/// Top-level application error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("CAN bus error: {0}")]
    Can(#[from] CanError),

    #[error("MQTT error: {0}")]
    Mqtt(#[from] MqttError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("OBD-II error: {0}")]
    Obd(#[from] ObdError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Shutdown requested")]
    Shutdown,

    #[error("Initialization failed: {0}")]
    Initialization(String),

    #[error("Fatal error: {0}")]
    Fatal(String),
}

/// Result type alias for application operations
pub type Result<T> = std::result::Result<T, AppError>;

/// Result type alias for CAN operations
pub type CanResult<T> = std::result::Result<T, CanError>;

/// Result type alias for MQTT operations
pub type MqttResult<T> = std::result::Result<T, MqttError>;

/// Result type alias for configuration operations
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

/// Result type alias for OBD operations
pub type ObdResult<T> = std::result::Result<T, ObdError>;

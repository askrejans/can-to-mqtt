//! Command-line interface definition using clap
//!
//! Provides subcommands for running the application in different modes

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "can-to-mqtt",
    version,
    about = "CAN bus to MQTT bridge for OBD-II vehicle telemetry",
    long_about = "A read-only Linux SocketCAN bridge with local setup, supported-PID discovery and verified passive telemetry.\n\
                  Supports daemon mode, health checks, and Prometheus metrics."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to configuration file
    #[arg(short, long, global = true)]
    pub config: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, global = true, default_value = "info")]
    pub log_level: String,

    /// Show the local setup UI without opening a CAN socket
    #[arg(long, global = true)]
    pub web_only: bool,

    /// Disable terminal display output
    #[arg(long, global = true)]
    pub no_display: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run the CAN-to-MQTT bridge (default mode)
    Run {
        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
    },

    /// Run as a daemon/service
    Daemon {
        /// Path to PID file
        #[arg(short, long, default_value = "/var/run/can-to-mqtt.pid")]
        pid_file: String,

        /// Enable systemd integration (sd-notify)
        #[arg(long)]
        systemd: bool,
    },

    /// Validate configuration file
    ValidateConfig,

    /// Test CAN and MQTT connections
    TestConnection,

    /// Show version information
    Version,
}

impl Cli {
    /// Get the effective command (Run is default if none specified)
    pub fn effective_command(&self) -> &Commands {
        self.command
            .as_ref()
            .unwrap_or(&Commands::Run { foreground: true })
    }
}

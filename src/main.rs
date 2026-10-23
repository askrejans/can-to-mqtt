//! Local read-only SocketCAN telemetry service and setup UI.
use can_to_mqtt::{
    cli::{Cli, Commands},
    config::{AppConfig, load_configuration},
    error::AppError,
};
use clap::Parser;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let cli = Cli::parse();
    let _ = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cli.log_level)))
        .with(tracing_subscriber::fmt::layer())
        .try_init();
    if matches!(cli.effective_command(), Commands::Version) {
        println!("can-to-mqtt {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let mut config = load_configuration(cli.config.as_deref())?;
    if cli.no_display {
        config.no_display = true
    }
    if matches!(cli.effective_command(), Commands::ValidateConfig) {
        println!("Configuration valid · {} · {}", config.mode, config.base());
        return Ok(());
    }
    if matches!(cli.effective_command(), Commands::TestConnection) {
        return test_connection(&config);
    }
    info!(version=env!("CARGO_PKG_VERSION"),mode=%config.mode,interface=%config.can_interface,"Starting read-only CAN telemetry");
    if let Commands::Daemon { pid_file, systemd } = cli.effective_command() {
        std::fs::write(pid_file, std::process::id().to_string())?;
        if *systemd {
            let _ = sd_notify::notify(false, &[sd_notify::NotifyState::Ready]);
        }
        let result = can_to_mqtt::runtime::run(config, cli.web_only).await;
        let _ = std::fs::remove_file(pid_file);
        if *systemd {
            let _ = sd_notify::notify(false, &[sd_notify::NotifyState::Stopping]);
        }
        return result;
    }
    can_to_mqtt::runtime::run(config, cli.web_only).await
}
fn test_connection(config: &AppConfig) -> Result<(), AppError> {
    #[cfg(target_os = "linux")]
    {
        use socketcan::Socket;
        socketcan::CanSocket::open(&config.can_interface)
            .map_err(|e| AppError::Initialization(format!("Could not open CAN interface: {e}")))?;
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = config;
        Err(AppError::Initialization("Native SocketCAN acquisition requires Linux; use a separate ELM adapter service on this platform".into()))
    }
    #[cfg(target_os = "linux")]
    {
        let client = can_to_mqtt::mqtt_handler::setup_mqtt(config)?;
        let _ = client.disconnect(None);
        println!(
            "CAN receive socket and MQTT connected. No ECU values or phone delivery have been verified."
        );
        Ok(())
    }
}

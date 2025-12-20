# CAN to MQTT

A Rust application that interfaces with a vehicle's CAN bus to retrieve On-Board Diagnostics (OBD-II) data and publish it to an MQTT broker. It is designed for real-time vehicle data monitoring and diagnostics with comprehensive logging, metrics, and health checks.

## Overview

The `can-to-mqtt` application connects to a vehicle's CAN bus via socketcan, sends OBD-II requests at configurable intervals, processes responses to extract vehicle parameters (engine RPM, speed, temperature, etc.), and publishes the data to an MQTT broker. The application features graceful shutdown, automatic reconnection, and can run as a systemd daemon.

## Features

- **OBD-II Protocol**: Reads 50+ PIDs including engine, fuel, temperature, and diagnostic parameters
- **MQTT Publishing**: Change-detection publishing with configurable QoS and retained messages
- **Prometheus Metrics**: HTTP endpoint (`:9090/metrics`) for monitoring
- **Health Checks**: `/health`, `/ready`, `/live` endpoints for Kubernetes-style probes
- **Systemd Integration**: Runs as a daemon with watchdog support

## Project Structure

```
can-to-mqtt
├── src
│   ├── main.rs           # Application entry point and main loop
│   ├── lib.rs            # Library root, exporting modules
│   ├── cli.rs            # CLI argument parsing with clap
│   ├── config.rs         # Configuration loading and validation
│   ├── error.rs          # Comprehensive error types
│   ├── metrics.rs        # Prometheus metrics and HTTP server
│   ├── mqtt_handler.rs   # MQTT connection and publishing
│   ├── constants.rs      # OBD-II constants
│   ├── vehicle           # Module for vehicle data management
│   │   ├── mod.rs        # Vehicle module definitions
│   │   └── data.rs       # VehicleData struct (85 fields)
│   ├── obd               # Module for OBD-II communication
│   │   ├── mod.rs        # OBD module definitions
│   │   ├── request.rs    # OBD request functions
│   │   └── response.rs   # OBD response parsing (79 PIDs)
│   └── display           # Module for terminal display
│       ├── mod.rs        # Display module definitions
│       └── table.rs      # Table formatting
├── contrib
│   ├── can-to-mqtt.service  # Systemd unit file
│   └── can-to-mqtt.env      # Environment file template
├── Cargo.toml            # Project dependencies and configuration
└── README.md             # This file
```

## Setup Instructions

### Prerequisites

- **Rust toolchain** (1.70+): Install from [rustup.rs](https://rustup.rs/)
- **Linux system** with CAN bus support (socketcan)
- **CAN interface** configured (e.g., `can0`)
- **MQTT broker** running (e.g., Mosquitto)

### Build and Install

1. Clone the repository:
   ```bash
   git clone https://github.com/askrejans/can-to-mqtt.git
   cd can-to-mqtt
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Install system-wide (optional):
   ```bash
   sudo cp target/release/can-to-mqtt /usr/local/bin/
   sudo cp contrib/can-to-mqtt.service /etc/systemd/system/
   sudo mkdir -p /etc/g86-car-telemetry
   sudo cp src/example.settings.toml /etc/g86-car-telemetry/can-to-mqtt.toml
   sudo systemctl daemon-reload
   ```

## Configuration

Create a configuration file at `/etc/g86-car-telemetry/can-to-mqtt.toml` or `./settings.toml`:

```toml
can_interface = "can0"
mqtt_host = "localhost"
mqtt_port = 1883
mqtt_base_topic = "/GOLF86/ECU/"

# Optional settings (see src/example.settings.toml for all options)
mqtt_username = "user"
mqtt_password = "pass"
log_level = "info"
poll_interval_high_freq_ms = 20
poll_interval_regular_ms = 200
prometheus_enabled = true
prometheus_port = 9090
```

Environment variables can override any setting using `CAN2MQTT_` prefix:
```bash
export CAN2MQTT_MQTT_HOST=mqtt.example.com
export CAN2MQTT_LOG_LEVEL=debug
```

## Usage

### Running the Application

```bash
# Run in foreground with terminal display
can-to-mqtt run --foreground

# Run as daemon
can-to-mqtt daemon

# With custom config
can-to-mqtt --config /path/to/config.toml run

# Validate configuration
can-to-mqtt validate-config

# Test CAN and MQTT connections
can-to-mqtt test-connection
```

### Systemd Service

```bash
# Start the service
sudo systemctl start can-to-mqtt

# Enable at boot
sudo systemctl enable can-to-mqtt

# Check status
sudo systemctl status can-to-mqtt

# View logs
sudo journalctl -u can-to-mqtt -f
```

## How It Works

Once running, the application:

1. Opens the CAN bus interface and connects to the MQTT broker
2. Sends OBD-II requests for configured PIDs at two frequencies:
   - **High-frequency** (20ms): RPM, speed, engine load, timing advance, throttle position
   - **Regular frequency** (200ms): 50+ other parameters (temperatures, pressures, fuel data, etc.)
3. Receives and parses OBD-II responses from the CAN bus
4. Publishes changed values to MQTT topics (e.g., `/GOLF86/ECU/RPM`, `/GOLF86/ECU/VSS`)
5. Updates terminal display (if enabled) and records metrics
6. Handles reconnections automatically if MQTT or CAN connections drop

### MQTT Topics

Topics follow the pattern `{mqtt_base_topic}/{PARAMETER}`:
- `/GOLF86/ECU/RPM` - Engine RPM
- `/GOLF86/ECU/VSS` - Vehicle speed
- `/GOLF86/ECU/CLT` - Coolant temperature
- `/GOLF86/ECU/TPS` - Throttle position
- And 50+ more parameters...

Messages are published with retained flag and only when values change.

## Monitoring

The application exposes Prometheus metrics on port 9090 (configurable):

```bash
# View metrics
curl http://localhost:9090/metrics

# Check health
curl http://localhost:9090/health

# Readiness probe (for Kubernetes)
curl http://localhost:9090/ready
```

Key metrics include CAN frame counts, MQTT publish rates, OBD request/response times, and connection status.

## Troubleshooting

### CAN Interface Issues
```bash
# Check interface status
ip link show can0

# Bring up CAN interface with 500kbit/s
sudo ip link set can0 up type can bitrate 500000

# Monitor raw CAN traffic
candump can0
```

### MQTT Issues
```bash
# Test broker connectivity
mosquitto_sub -h localhost -p 1883 -t "/GOLF86/ECU/#" -v

# Check connection in logs
sudo journalctl -u can-to-mqtt | grep -i mqtt
```

### Check Application Status
```bash
# View metrics
curl http://localhost:9090/metrics | grep -E "can_|mqtt_|obd_"

# Check health
curl http://localhost:9090/health | jq
```

## Dependencies

This project uses the following main dependencies:

- `socketcan` - CAN bus communication (Linux only)
- `tokio` - Async runtime
- `paho-mqtt` - MQTT client
- `clap` - CLI argument parsing
- `tracing` - Structured logging
- `metrics` - Prometheus metrics
- `hyper` - HTTP server for metrics
- See `Cargo.toml` for complete list

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions welcome! Please open an issue or submit a pull request.

## Acknowledgments

- Built with [socketcan-rs](https://github.com/socketcan-rs/socketcan-rs)
- MQTT via [paho.mqtt.rust](https://github.com/eclipse/paho.mqtt.rust)
- OBD-II protocol based on SAE J1979 standard

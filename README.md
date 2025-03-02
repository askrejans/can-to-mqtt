# README.md

# CAN to MQTT

This project is a Rust application that interfaces with a vehicle's CAN bus to retrieve On-Board Diagnostics (OBD) data and publish it to an MQTT broker. It is designed to provide real-time vehicle data monitoring and diagnostics.

## Overview

The `can-to-mqtt` application connects to a vehicle's CAN bus, sends OBD requests, and processes the responses to extract various vehicle parameters such as engine RPM, vehicle speed, coolant temperature, and more. The retrieved data is then formatted and displayed in a structured manner.

## Project Structure

```
can-to-mqtt
├── src
│   ├── main.rs          # Entry point of the application
│   ├── lib.rs           # Library root, exporting main modules
│   ├── vehicle          # Module for vehicle data management
│   │   ├── mod.rs       # Vehicle module definitions
│   │   └── data.rs      # VehicleData struct definition
│   ├── obd              # Module for OBD communication
│   │   ├── mod.rs       # OBD module definitions
│   │   ├── request.rs    # OBD request functions
│   │   └── response.rs   # OBD response parsing functions
│   ├── display          # Module for displaying vehicle data
│   │   ├── mod.rs       # Display module definitions
│   │   └── table.rs     # Table management for displaying data
│   └── constants.rs     # Constants used throughout the application
├── Cargo.toml           # Project configuration file
└── README.md            # Project documentation
```

## Setup Instructions

1. Clone the repository:
   ```
   git clone <repository-url>
   cd can-to-mqtt
   ```

2. Build the project:
   ```
   cargo build
   ```

3. Run the application:
   ```
   cargo run
   ```

## Usage

Once the application is running, it will connect to the vehicle's CAN bus and start sending OBD requests. The retrieved data will be displayed in the console in a structured format.

## Dependencies

This project uses the following dependencies:
- `socketcan`: For CAN bus communication.
- `tokio`: For asynchronous programming.
- `futures-util`: For working with asynchronous streams.
- `prettytable-rs`: For formatting and displaying tables in the console.

## License

This project is licensed under the MIT License. See the LICENSE file for more details.# can-to-mqtt

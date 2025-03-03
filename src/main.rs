use can_to_mqtt::vehicle::data::VehicleData;
use futures_util::StreamExt;
use socketcan::{
    CanFrame,
    embedded_can::{Frame, Id, StandardId},
    tokio::CanSocket,
};

use can_to_mqtt::constants::OBD_RESPONSE_ID;
use can_to_mqtt::display::display_vehicle_data;
use can_to_mqtt::obd::request::send_obd_request;
use can_to_mqtt::obd::response::parse_obd_response;
use tokio;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut socket_rx =
        CanSocket::open("can0").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let socket_tx =
        CanSocket::open("can0").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut vehicle_data = VehicleData::default();

    // Separate high-frequency PIDs
    let high_freq_pids = [
        (0x0C, "Engine RPM"),
        (0x0D, "Vehicle speed"),
        (0x04, "Engine load"),
        (0x0E, "Timing advance"),
        (0x11, "Throttle position"),
    ];

    // Regular frequency PIDs (excluding RPM and speed)
    let regular_pids = [
        // Mode 01 PIDs
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
    ];

    let mut regular_cycle_counter = 0;

    loop {
        // Request high-frequency PIDs every cycle
        for (pid, desc) in high_freq_pids.iter() {
            if let Err(e) = send_request(&socket_tx, *pid).await {
                eprintln!("Error sending request for {}: {}", desc, e);
            }
        }

        // Request regular PIDs every 10 cycles (200ms / 20ms = 10)
        if regular_cycle_counter == 0 {
            for (pid, desc) in regular_pids.iter() {
                if let Err(e) = send_request(&socket_tx, *pid).await {
                    eprintln!("Error sending request for {}: {}", desc, e);
                }
            }
        }

        let timeout = tokio::time::sleep(tokio::time::Duration::from_millis(20));
        tokio::pin!(timeout);

        let expected_responses = if regular_cycle_counter == 0 {
            high_freq_pids.len() + regular_pids.len()
        } else {
            high_freq_pids.len()
        };

        let mut responses_received = 0;
        while responses_received < expected_responses {
            tokio::select! {
                frame = socket_rx.next() => {
                    match frame {
                        Some(Ok(frame)) => {
                            if let CanFrame::Data(frame) = frame {
                                if frame.id() == Id::Standard(StandardId::new(OBD_RESPONSE_ID).unwrap()) {
                                    parse_obd_response(&frame, &mut vehicle_data);
                                    responses_received += 1;
                                }
                            }
                        }
                        Some(Err(e)) => eprintln!("Error reading frame: {}", e),
                        None => break,
                    }
                }
                _ = &mut timeout => {
                    break;
                }
            }
        }

        display_vehicle_data(&vehicle_data);

        // Update cycle counter
        regular_cycle_counter = (regular_cycle_counter + 1) % 10;
    }
}

async fn send_request(socket: &CanSocket, pid: u8) -> std::io::Result<()> {
    send_obd_request(socket, pid)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

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
    let pids = [
        (0x04, "Engine load"),
        (0x05, "Coolant temperature"),
        (0x0A, "Fuel pressure"),
        (0x0B, "Intake manifold pressure"),
        (0x0C, "Engine RPM"),
        (0x0D, "Vehicle speed"),
        (0x0E, "Timing advance"),
        (0x0F, "Intake air temperature"),
        (0x10, "MAF sensor"),
        (0x11, "Throttle position"),
        (0x14, "O2 Sensor Voltage"),
        (0x2F, "Fuel Level"),
        (0x33, "Barometric pressure"),
        (0x46, "Ambient temperature"),
        (0x23, "Fuel rail pressure"),
        (0x2C, "Commanded EGR"),
        (0x2D, "EGR Error"),
        (0x06, "Short term fuel trim Bank 1"),
        (0x07, "Long term fuel trim Bank 1"),
    ];

    const BATCH_SIZE: usize = 5;

    loop {
        // Process PIDs in smaller batches
        for chunk in pids.chunks(BATCH_SIZE) {
            // Send requests for current batch
            for (pid, desc) in chunk.iter() {
                if let Err(e) = send_request(&socket_tx, *pid).await {
                    eprintln!("Error sending request for {}: {}", desc, e);
                    continue;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }

            // Wait for responses for current batch
            let timeout = tokio::time::sleep(tokio::time::Duration::from_millis(200));
            tokio::pin!(timeout);

            let mut responses_received = 0;
            while responses_received < chunk.len() {
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
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            display_vehicle_data(&vehicle_data);

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    async fn send_request(socket: &CanSocket, pid: u8) -> std::io::Result<()> {
        send_obd_request(socket, pid)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

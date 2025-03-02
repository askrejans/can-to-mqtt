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

    loop {
        // Send OBD requests for all parameters
        send_request(&socket_tx, 0x04).await?; // Engine load
        send_request(&socket_tx, 0x05).await?; // Coolant temperature
        send_request(&socket_tx, 0x0A).await?; // Fuel pressure
        send_request(&socket_tx, 0x0B).await?; // Intake manifold pressure
        send_request(&socket_tx, 0x0C).await?; // Engine RPM
        send_request(&socket_tx, 0x0D).await?; // Vehicle speed
        send_request(&socket_tx, 0x0E).await?; // Timing advance
        send_request(&socket_tx, 0x0F).await?; // Intake air temperature
        send_request(&socket_tx, 0x10).await?; // MAF sensor
        send_request(&socket_tx, 0x11).await?; // Throttle position
        send_request(&socket_tx, 0x14).await?; // O2 Sensor Voltage
        send_request(&socket_tx, 0x2F).await?; // Fuel Level
        send_request(&socket_tx, 0x33).await?; // Barometric pressure
        send_request(&socket_tx, 0x46).await?; // Ambient temperature
        send_request(&socket_tx, 0x23).await?; // Fuel rail pressure
        send_request(&socket_tx, 0x2C).await?; // Commanded EGR
        send_request(&socket_tx, 0x2D).await?; // EGR Error
        send_request(&socket_tx, 0x06).await?; // Short term fuel trim Bank 1
        send_request(&socket_tx, 0x07).await?; // Long term fuel trim Bank 1

        let timeout = tokio::time::sleep(tokio::time::Duration::from_millis(300));
        tokio::pin!(timeout);

        for _ in 0..21 {
            tokio::select! {
                frame = socket_rx.next() => {
                    if let Some(Ok(frame)) = frame {
                        if let CanFrame::Data(frame) = frame {
                            if frame.id() == Id::Standard(StandardId::new(OBD_RESPONSE_ID).unwrap()) {
                                parse_obd_response(&frame, &mut vehicle_data);
                            }
                        }
                    }
                }
                _ = &mut timeout => {
                    break;
                }
            }
        }

        display_vehicle_data(&vehicle_data);

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    async fn send_request(socket: &CanSocket, pid: u8) -> std::io::Result<()> {
        send_obd_request(socket, pid)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

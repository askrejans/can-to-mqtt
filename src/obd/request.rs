use socketcan::{
    CanFrame, Result,
    embedded_can::{Frame, Id, StandardId},
    tokio::CanSocket,
};
use std::io::Error as IoError;

const OBD_REQUEST_ID: u16 = 0x7DF;

pub async fn send_obd_request(socket: &CanSocket, pid: u8) -> Result<()> {
    let standard_id = StandardId::new(OBD_REQUEST_ID)
        .ok_or_else(|| IoError::new(std::io::ErrorKind::InvalidInput, "Invalid CAN ID"))?;

    let frame = CanFrame::new(Id::Standard(standard_id), &[0x02, 0x01, pid, 0, 0, 0, 0, 0])
        .expect("Failed to create CAN frame");

    socket.write_frame(frame).await?;
    Ok(())
}

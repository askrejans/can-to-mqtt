//! Decode only an actually received Mode 01 PID. Never publish default zero fields.
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn packet(bytes: &[u8], boot: &str, sequence: u64, captured: SystemTime) -> Option<Value> {
    // Single-frame ISO-TP, positive Mode 01 response, declared payload fits the frame.
    if bytes.len() < 4 || bytes[0] < 3 || bytes[0] > 7 || usize::from(bytes[0]) + 1 > bytes.len() || bytes[1] != 0x41 { return None; }
    let a = f64::from(bytes[3]);
    let word = || if bytes[0] >= 4 { Some(a * 256.0 + f64::from(bytes[4])) } else { None };
    let (name, value) = match bytes[2] {
        0x04 => ("engineLoad", a * 100.0 / 255.0),
        0x05 => ("coolantC", a - 40.0),
        0x0A => ("fuelPressureKpa", a * 3.0),
        0x0B => ("manifoldKpa", a),
        0x0C => ("rpm", word()? / 4.0),
        0x0D => ("ecuSpeedKmh", a),
        0x0E => ("ignitionDeg", a / 2.0 - 64.0),
        0x0F => ("intakeC", a - 40.0),
        0x10 => ("massAirFlowGps", word()? / 100.0),
        0x11 => ("throttle", a * 100.0 / 255.0),
        0x2F => ("fuelLevel", a * 100.0 / 255.0),
        0x42 => ("batteryV", word()? / 1000.0),
        0x5C => ("oilC", a - 40.0),
        _ => return None,
    };
    Some(json!({"schema":1,"source":"obd2/mode01","partial":true,
        "bootId":boot,"sequence":sequence,
        "timestampMs":captured.duration_since(UNIX_EPOCH).ok()?.as_millis() as u64,
        "channels":{name:value}}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_frame_and_preserves_units() {
        let now = UNIX_EPOCH + std::time::Duration::from_secs(100);
        let p = packet(&[4,0x41,0x0C,0x1f,0x40,0,0,0], "b", 1, now).unwrap();
        assert_eq!(p["channels"]["rpm"], 2000.0);
        assert_eq!(p["channels"].as_object().unwrap().len(), 1);
        assert_eq!(packet(&[3,0x41,0x0A,255], "b",2,now).unwrap()["channels"]["fuelPressureKpa"],765.0);
        assert_eq!(packet(&[3,0x41,0x11,255], "b",3,now).unwrap()["channels"]["throttle"],100.0);
        for bytes in [&[3,0x41,0x0C,0][..], &[4,0x41,0x0C,0][..], &[3,0x7f,0x11,20][..], &[3,0x41,0xff,0][..]] {
            assert!(packet(bytes,"b",1,now).is_none());
        }
    }
}

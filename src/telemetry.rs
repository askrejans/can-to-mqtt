//! Canonical telemetry is built only from an actually received positive PID.
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};

pub const PIDS: &[u8] = &[
    0x04, 0x05, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x24, 0x2F, 0x33, 0x42, 0x46, 0x5C,
];
pub fn captured_ms(captured: SystemTime) -> Option<u64> {
    u64::try_from(captured.duration_since(UNIX_EPOCH).ok()?.as_millis()).ok()
}
pub fn packet(bytes: &[u8], boot: &str, sequence: u64, captured: SystemTime) -> Option<Value> {
    let payload = positive_payload(bytes)?;
    let pid = *payload.get(1)?;
    let data = payload.get(2..)?;
    let a = f64::from(*data.first()?);
    let word = || Some(a * 256.0 + f64::from(*data.get(1)?));
    let (name, value) = match pid {
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
        0x24 if data.len() >= 4 => ("lambda", word()? / 32768.0),
        0x2F => ("fuelLevel", a * 100.0 / 255.0),
        0x33 => ("baroKpa", a),
        0x42 => ("batteryV", word()? / 1000.0),
        0x46 => ("ambientC", a - 40.0),
        0x5C => ("oilC", a - 40.0),
        _ => return None,
    };
    if !valid_channel(name, value) {
        return None;
    }
    Some(envelope(
        "obd2/mode01",
        boot,
        sequence,
        captured_ms(captured)?,
        json!({name:value}),
    ))
}
pub fn positive_payload(bytes: &[u8]) -> Option<&[u8]> {
    let length = usize::from(*bytes.first()?);
    if !(2..=7).contains(&length) || bytes.len() < length + 1 || bytes[1] != 0x41 {
        return None;
    };
    Some(&bytes[1..=length])
}
pub fn envelope(source: &str, boot: &str, sequence: u64, timestamp: u64, channels: Value) -> Value {
    json!({"schema":1,"source":source,"partial":true,"bootId":boot,"sequence":sequence,"timestampMs":timestamp,"channels":channels})
}
pub fn valid_channel(name: &str, value: f64) -> bool {
    channel_range(name).is_some_and(|(low, high, integer)| {
        value.is_finite() && value >= low && value <= high && (!integer || value.fract() == 0.0)
    })
}
/// Canonical ranges shared with the phone registry; unknown names are rejected.
pub fn channel_range(name: &str) -> Option<(f64, f64, bool)> {
    Some(match name {
        "rpm" => (0.0, 20000.0, false),
        "engineLoad" => (0.0, 100.0, false),
        "ignitionDeg" => (-100.0, 100.0, false),
        "dwellMs" => (0.0, 50.0, false),
        "syncLossCount" => (0.0, 255.0, true),
        "throttle" => (0.0, 100.0, false),
        "brake" => (0.0, 100.0, false),
        "brakePressureKpa" => (0.0, 20000.0, false),
        "brakeSwitch" => (0.0, 1.0, true),
        "clutchSwitch" => (0.0, 1.0, true),
        "gear" => (-1.0, 12.0, true),
        "coolantC" => (-50.0, 250.0, false),
        "oilC" => (-50.0, 250.0, false),
        "intakeC" => (-50.0, 250.0, false),
        "transmissionC" => (-50.0, 250.0, false),
        "differentialC" => (-50.0, 250.0, false),
        "fuelC" => (-50.0, 250.0, false),
        "oilPressureKpa" => (0.0, 5000.0, false),
        "manifoldKpa" => (0.0, 1000.0, false),
        "boostKpa" => (-100.0, 550.0, false),
        "fuelPressureKpa" => (0.0, 5000.0, false),
        "lambda" => (0.4, 3.0, false),
        "lambda2" => (0.4, 3.0, false),
        "afr" => (5.0, 30.0, false),
        "afrTarget" => (5.0, 30.0, false),
        "veCurrent" => (0.0, 255.0, false),
        "injectorPwMs" => (0.0, 100.0, false),
        "fuelLevel" => (0.0, 100.0, false),
        "massAirFlowGps" => (0.0, 1000.0, false),
        "batteryV" => (0.0, 36.0, false),
        "busV" => (0.0, 36.0, false),
        "currentA" => (-100.0, 100.0, false),
        "powerW" => (-2000.0, 2000.0, false),
        "batterySocPct" => (0.0, 100.0, false),
        "ecuSpeedKmh" => (0.0, 500.0, false),
        "wheelSpeedFlKmh" => (0.0, 500.0, false),
        "wheelSpeedFrKmh" => (0.0, 500.0, false),
        "wheelSpeedRlKmh" => (0.0, 500.0, false),
        "wheelSpeedRrKmh" => (0.0, 500.0, false),
        "lateralG" => (-10.0, 10.0, false),
        "longitudinalG" => (-10.0, 10.0, false),
        "verticalG" => (-16.0, 16.0, false),
        "combinedG" => (0.0, 16.0, false),
        "peakG" => (0.0, 16.0, false),
        "yawRateDegS" => (-2000.0, 2000.0, false),
        "rollRateDegS" => (-2000.0, 2000.0, false),
        "pitchRateDegS" => (-2000.0, 2000.0, false),
        "leanDeg" => (-90.0, 90.0, false),
        "bankDeg" => (-90.0, 90.0, false),
        "tiltDeg" => (0.0, 180.0, false),
        "gpsSpeedKmh" => (0.0, 500.0, false),
        "maxSpeedKmh" => (0.0, 500.0, false),
        "altitudeM" => (-500.0, 9000.0, false),
        "accuracyM" => (0.0, 1000.0, false),
        "satellitesUsed" => (0.0, 64.0, true),
        "satellitesInView" => (0.0, 128.0, true),
        "hdop" => (0.0, 100.0, false),
        "vdop" => (0.0, 100.0, false),
        "pdop" => (0.0, 100.0, false),
        "rateOfTurnDps" => (-2000.0, 2000.0, false),
        "yawRateDps" => (-2000.0, 2000.0, false),
        "distanceM" => (0.0, 10000000.0, false),
        "ambientC" => (-50.0, 80.0, false),
        "humidityPct" => (0.0, 100.0, false),
        "baroKpa" => (50.0, 120.0, false),
        "baroAltitudeM" => (-500.0, 9000.0, false),
        "lightLux" => (0.0, 200000.0, false),
        "pm25" => (0.0, 1000.0, false),
        "pm10" => (0.0, 1000.0, false),
        "aqiPm25" => (0.0, 500.0, false),
        "aqiPm10" => (0.0, 500.0, false),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn mode01_preserves_values_and_never_zero_fills() {
        let at = UNIX_EPOCH + Duration::from_secs(123);
        let p = packet(&[4, 0x41, 0x0C, 0x1F, 0x40], "boot", 8, at).unwrap();
        assert_eq!(p["channels"], json!({"rpm":2000.0}));
        assert_eq!(p["timestampMs"], 123000);
        assert_eq!(p["sequence"], 8);
        assert_eq!(p["source"], "obd2/mode01");
        for bytes in [
            &[3, 0x41, 0x0C, 1][..],
            &[4, 0x41, 0x0C, 1][..],
            &[3, 0x7F, 1, 1][..],
            &[3, 0x41, 0xFF, 0][..],
            &[4, 0x41, 0x42, 0xFF, 0xFF][..],
        ] {
            assert!(packet(bytes, "boot", 1, at).is_none());
        }
    }
    #[test]
    fn canonical_ranges_reject_unknown_nonfinite_and_fractional_gears() {
        assert!(valid_channel("gear", -1.0));
        assert!(!valid_channel("gear", 1.5));
        assert!(!valid_channel("rpm", f64::NAN));
        assert!(!valid_channel("batteryV", 40.0));
        assert!(!valid_channel("inventedSignal", 1.0));
        assert!(packet(&[6, 0x41, 0x24, 0x80, 0, 0, 0], "b", 1, UNIX_EPOCH).is_some());
    }
}

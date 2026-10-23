//! Explicit engineer-verified signal definitions; no speculative vehicle profiles.
use crate::telemetry::{envelope, valid_channel};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PassiveSignal {
    pub channel: String,
    pub can_id: u32,
    pub extended: bool,
    pub start_bit: u8,
    pub bit_length: u8,
    pub byte_order: String,
    pub signed: bool,
    pub factor: f64,
    pub offset: f64,
    pub source_reference: String,
    pub verified: bool,
}
impl Default for PassiveSignal {
    fn default() -> Self {
        Self {
            channel: String::new(),
            can_id: 0,
            extended: false,
            start_bit: 0,
            bit_length: 0,
            byte_order: "little".into(),
            signed: false,
            factor: 1.0,
            offset: 0.0,
            source_reference: String::new(),
            verified: false,
        }
    }
}
impl PassiveSignal {
    pub fn validate(&self) -> Result<(), String> {
        if !self.verified
            || self.source_reference.trim().is_empty()
            || self.source_reference.len() > 240
            || self.source_reference.chars().any(char::is_control)
        {
            return Err("Passive signals need an explicit verified source reference".into());
        }
        if crate::telemetry::channel_range(&self.channel).is_none() {
            return Err("Unknown canonical channel".into());
        }
        if self.can_id > if self.extended { 0x1FFF_FFFF } else { 0x7FF }
            || self.bit_length == 0
            || self.bit_length > 32
            || u16::from(self.start_bit) + u16::from(self.bit_length) > 64
        {
            return Err("Invalid CAN ID or signal bit range".into());
        }
        if !["little", "big"].contains(&self.byte_order.as_str())
            || !self.factor.is_finite()
            || self.factor == 0.0
            || !self.offset.is_finite()
        {
            return Err("Invalid signal byte order or scale".into());
        };
        Ok(())
    }
    pub fn decode(&self, id: u32, extended: bool, bytes: &[u8]) -> Option<f64> {
        if self.can_id != id
            || self.extended != extended
            || self.validate().is_err()
            || usize::from(self.start_bit) + usize::from(self.bit_length) > bytes.len() * 8
        {
            return None;
        }
        let mut raw = 0u64;
        for i in 0..self.bit_length {
            let bit = usize::from(self.start_bit) + usize::from(i);
            let value = if self.byte_order == "little" {
                (bytes[bit / 8] >> (bit % 8)) & 1
            } else {
                (bytes[bit / 8] >> (7 - bit % 8)) & 1
            };
            if self.byte_order == "little" {
                raw |= u64::from(value) << i
            } else {
                raw = (raw << 1) | u64::from(value)
            }
        }
        let signed = if self.signed && raw & (1u64 << (self.bit_length - 1)) != 0 {
            raw as i64 - (1i64 << self.bit_length)
        } else {
            raw as i64
        };
        let value = signed as f64 * self.factor + self.offset;
        valid_channel(&self.channel, value).then_some(value)
    }
}
pub fn packet(
    signals: &[PassiveSignal],
    profile: &str,
    identity: crate::session::Identity,
    bytes: &[u8],
    boot: &str,
    sequence: u64,
    timestamp: u64,
) -> Option<Value> {
    let mut channels = serde_json::Map::new();
    for signal in signals {
        if let Some(value) = signal.decode(identity.id, identity.extended, bytes) {
            channels.insert(signal.channel.clone(), json!(value));
        }
    }
    if channels.is_empty() {
        return None;
    };
    Some(envelope(
        &format!("sensors/can/{profile}"),
        boot,
        sequence,
        timestamp,
        Value::Object(channels),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Identity;
    fn signal() -> PassiveSignal {
        PassiveSignal {
            channel: "rpm".into(),
            can_id: 0x123,
            start_bit: 0,
            bit_length: 16,
            factor: 0.25,
            source_reference: "Recorded test fixture, independently specified RPM at bytes 0–1"
                .into(),
            verified: true,
            ..PassiveSignal::default()
        }
    }
    #[test]
    fn decodes_only_verified_matching_frame_and_preserves_units() {
        let s = signal();
        assert_eq!(s.decode(0x123, false, &[0x40, 0x1F]), Some(2000.0));
        assert_eq!(s.decode(0x124, false, &[0x40, 0x1F]), None);
        assert_eq!(s.decode(0x123, true, &[0x40, 0x1F]), None);
        assert_eq!(s.decode(0x123, false, &[0x40]), None);
        let mut big = s.clone();
        big.byte_order = "big".into();
        assert_eq!(big.decode(0x123, false, &[0x1F, 0x40]), Some(2000.0));
        let p = packet(
            &[s],
            "can",
            Identity {
                id: 0x123,
                extended: false,
            },
            &[0x40, 0x1F],
            "boot",
            3,
            123,
        )
        .unwrap();
        assert_eq!(p["source"], "sensors/can/can");
        assert_eq!(p["channels"], json!({"rpm":2000.0}));
        assert_eq!(p["timestampMs"], 123);
    }
    #[test]
    fn handles_signed_bits_and_rejects_missing_provenance_or_invalid_values() {
        let mut s = signal();
        s.channel = "ignitionDeg".into();
        s.signed = true;
        s.factor = 1.0;
        assert_eq!(s.decode(0x123, false, &[0xFE, 0xFF]), Some(-2.0));
        s.verified = false;
        assert!(s.validate().is_err());
        s.verified = true;
        s.source_reference.clear();
        assert!(s.validate().is_err());
        s.source_reference = "fixture".into();
        s.bit_length = 33;
        assert!(s.validate().is_err());
        s.bit_length = 16;
        s.factor = f64::NAN;
        assert!(s.validate().is_err());
        s.factor = 0.0;
        assert!(s.validate().is_err());
        s.factor = 1000.0;
        assert_eq!(s.decode(0x123, false, &[0x10, 0]), None);
    }
}

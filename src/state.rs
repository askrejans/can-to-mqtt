use crate::config::AppConfig;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::Mutex,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
struct Reading {
    value: Value,
    at: Instant,
    captured: u64,
    source: String,
}
struct Inner {
    can: bool,
    mqtt: bool,
    frames: u64,
    samples: u64,
    published: u64,
    dropped: u64,
    error: String,
    selected: Value,
    pids: Vec<u8>,
    channels: BTreeMap<String, Reading>,
}
pub struct Shared {
    pub config: AppConfig,
    pub boot: String,
    pub metrics: std::sync::Arc<crate::metrics::HealthState>,
    inner: Mutex<Inner>,
}
impl Shared {
    pub fn new(config: AppConfig) -> Result<Self, String> {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).map_err(|_| "Could not create a boot identity")?;
        let boot = bytes.iter().map(|b| format!("{b:02x}")).collect();
        Ok(Self {
            config,
            boot,
            metrics: std::sync::Arc::new(crate::metrics::HealthState::new()),
            inner: Mutex::new(Inner {
                can: false,
                mqtt: false,
                frames: 0,
                samples: 0,
                published: 0,
                dropped: 0,
                error: String::new(),
                selected: Value::Null,
                pids: vec![],
                channels: BTreeMap::new(),
            }),
        })
    }
    pub fn can(&self, on: bool, error: &str) {
        crate::metrics::set_can_connected(on);
        if let Ok(mut value) = self.metrics.can_connected.try_write() {
            *value = on;
        }
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        s.can = on;
        s.error = error.into();
        if !on {
            s.channels.clear()
        }
    }
    pub fn mqtt(&self, on: bool) {
        crate::metrics::set_mqtt_connected(on);
        if let Ok(mut value) = self.metrics.mqtt_connected.try_write() {
            *value = on;
        }
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).mqtt = on;
    }
    pub fn frame(&self) {
        crate::metrics::record_can_frame_received();
        if let Ok(mut value) = self.metrics.last_can_frame.try_write() {
            *value = Some(SystemTime::now());
        }
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).frames += 1;
    }
    pub fn drop_packet(&self) {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).dropped += 1;
    }
    pub fn published(&self) {
        crate::metrics::record_mqtt_publish();
        if let Ok(mut value) = self.metrics.last_mqtt_publish.try_write() {
            *value = Some(SystemTime::now());
        }
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .published += 1;
    }
    pub fn discovery(&self, selected: Value, pids: Vec<u8>) {
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        s.selected = selected;
        s.pids = pids;
    }
    pub fn sample(&self, packet: &Value, now: Instant, wall: u64) -> bool {
        let Some(captured) = packet["timestampMs"].as_u64() else {
            return false;
        };
        if captured > wall.saturating_add(500) || wall.saturating_sub(captured) > 2000 {
            return false;
        }
        let Some(channels) = packet["channels"].as_object() else {
            return false;
        };
        if channels.is_empty() {
            return false;
        }
        let channels: Vec<_> = channels
            .iter()
            .filter(|(name, value)| {
                value
                    .as_f64()
                    .is_some_and(|v| crate::telemetry::valid_channel(name, v))
            })
            .collect();
        if channels.is_empty() {
            return false;
        }
        if let Ok(mut value) = self.metrics.last_telemetry.try_write() {
            *value = Some(SystemTime::now());
        }
        let source = packet["source"].as_str().unwrap_or("");
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        s.samples += 1;
        for (name, value) in channels {
            s.channels.insert(
                name.clone(),
                Reading {
                    value: value.clone(),
                    at: now,
                    captured,
                    source: source.into(),
                },
            );
        }
        true
    }
    pub fn snapshot(&self, now: Instant) -> Value {
        let s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let channels:Vec<Value>=s.channels.iter().filter_map(|(name,r)|{let age=now.saturating_duration_since(r.at).as_millis() as u64;if age>2000{return None};Some(json!({"name":name,"value":r.value,"ageMs":age,"timestampMs":r.captured,"source":r.source}))}).collect();
        json!({"schema":1,"version":env!("CARGO_PKG_VERSION"),"bridge":"can-to-mqtt","mode":self.config.mode,"interface":self.config.can_interface,"can_connected":s.can,"mqtt_connected":s.mqtt,"ready":s.can&&s.mqtt&&!channels.is_empty(),"frames":s.frames,"samples":s.samples,"published":s.published,"dropped":s.dropped,"error":s.error,"ecu":s.selected,"supported_pids":s.pids,"channels":channels,"bootId":self.boot})
    }
    pub fn health(&self, down: bool) -> Value {
        let v = self.snapshot(Instant::now());
        json!({"schema":1,"bridge":"can-to-mqtt","version":env!("CARGO_PKG_VERSION"),"bootId":self.boot,"timestampMs":wall_ms(),"state":if down||v["can_connected"]!=true{"down"}else if v["ready"]==true{"ok"}else{"degraded"},"channels":v["channels"],"errors":{"drops":v["dropped"]},"clockSynced":crate::health::clock_synced()})
    }
}
pub fn wall_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn live_state_requires_actual_fresh_channels_and_real_broker() {
        let s = Shared::new(AppConfig::default()).unwrap();
        let now = Instant::now();
        s.can(true, "");
        s.mqtt(true);
        assert_eq!(s.snapshot(now)["ready"], false);
        let p = json!({"timestampMs":1000,"source":"obd2/mode01","channels":{"rpm":2000}});
        assert!(s.sample(&p, now, 1000));
        assert_eq!(s.snapshot(now)["ready"], true);
        assert_eq!(
            s.snapshot(now + Duration::from_millis(2001))["ready"],
            false
        );
        assert_eq!(
            s.snapshot(now + Duration::from_millis(2001))["channels"],
            json!([])
        );
        s.mqtt(false);
        assert_eq!(s.snapshot(now)["ready"], false);
    }
    #[test]
    fn stale_future_and_invalid_channels_do_not_become_fresh() {
        let s = Shared::new(AppConfig::default()).unwrap();
        let now = Instant::now();
        for stamp in [0, 20_000] {
            assert!(!s.sample(
                &json!({"timestampMs":stamp,"source":"obd2/mode01","channels":{"rpm":2000}}),
                now,
                10_000
            ));
        }
        s.can(true, "");
        s.mqtt(true);
        s.sample(&json!({"timestampMs":10_000,"source":"obd2/mode01","channels":{"batteryV":1000,"unknown":5}}),now,10_000);
        assert_eq!(s.snapshot(now)["channels"], json!([]));
        assert_eq!(s.snapshot(now)["ready"], false);
    }
}

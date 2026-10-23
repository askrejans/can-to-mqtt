//! Bridge health for `<base>/health` (G86 car computer contract §2).
//!
//! Retained JSON, published every 5 s and at once when `state` or `link`
//! changes. `down` = CAN socket closed or no OBD reply for 10 s; `degraded` =
//! last packet older than 2 s or a CAN/publish error in the last 5 s;
//! otherwise `ok`. `timeouts` are counted but do not degrade: a car that does
//! not support a polled PID times out on every regular cycle.

use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::time::{Duration, Instant};

const PUBLISH_EVERY: Duration = Duration::from_secs(5);
const RATE_WINDOW: Duration = Duration::from_secs(5);
const FRESH: Duration = Duration::from_secs(2);
const REPLY_TIMEOUT: Duration = Duration::from_secs(10);
const ERROR_WINDOW: Duration = Duration::from_secs(5);

/// Error counters that degrade the state when they increase.
const DEGRADING: [&str; 2] = ["canErrors", "publishErrors"];

pub struct Health {
    boot_id: String,
    socket_open: bool,
    last_reply: Option<Instant>,
    last_packet: Option<Instant>,
    last_capture_ms: Option<u64>,
    published: VecDeque<Instant>,
    channels: HashMap<String, Instant>,
    errors: BTreeMap<&'static str, u64>,
    last_degrading_error: Option<Instant>,
    last_sent: Option<(Instant, &'static str, &'static str)>,
}

impl Health {
    pub fn new(boot_id: &str) -> Self {
        let errors = ["canErrors", "publishErrors", "timeouts"]
            .into_iter()
            .map(|k| (k, 0))
            .collect();
        Self {
            boot_id: boot_id.to_string(),
            socket_open: false,
            last_reply: None,
            last_packet: None,
            last_capture_ms: None,
            published: VecDeque::new(),
            channels: HashMap::new(),
            errors,
            last_degrading_error: None,
            last_sent: None,
        }
    }

    pub fn set_socket(&mut self, open: bool) {
        self.socket_open = open;
    }

    /// Any OBD response frame (supported or not) proves the ECU answers.
    pub fn on_reply(&mut self, now: Instant) {
        self.last_reply = Some(now);
    }

    /// A canonical telemetry packet was published.
    pub fn on_packet(&mut self, packet: &Value, now: Instant) {
        self.last_packet = Some(now);
        self.last_capture_ms = packet["timestampMs"].as_u64().or(self.last_capture_ms);
        self.published.push_back(now);
        if let Some(channels) = packet["channels"].as_object() {
            for key in channels.keys() {
                self.channels.insert(key.clone(), now);
            }
        }
    }

    pub fn on_error(&mut self, counter: &'static str, now: Instant) {
        *self.errors.entry(counter).or_default() += 1;
        if DEGRADING.contains(&counter) {
            self.last_degrading_error = Some(now);
        }
    }

    pub fn link(&self, now: Instant) -> &'static str {
        if !self.socket_open {
            "disconnected"
        } else if self
            .last_reply
            .is_some_and(|t| now.duration_since(t) <= REPLY_TIMEOUT)
        {
            "connected"
        } else {
            "connecting"
        }
    }

    pub fn state(&self, now: Instant) -> &'static str {
        if self.link(now) != "connected" {
            "down"
        } else if self
            .last_packet
            .is_none_or(|t| now.duration_since(t) > FRESH)
            || self
                .last_degrading_error
                .is_some_and(|t| now.duration_since(t) < ERROR_WINDOW)
        {
            "degraded"
        } else {
            "ok"
        }
    }

    /// The health JSON when it is due (every 5 s or on a state/link change).
    pub fn due(&mut self, now: Instant, wall_ms: u64, clock_synced: bool) -> Option<Value> {
        let (state, link) = (self.state(now), self.link(now));
        let due = match self.last_sent {
            None => true,
            Some((at, s, l)) => now.duration_since(at) >= PUBLISH_EVERY || s != state || l != link,
        };
        if !due {
            return None;
        }
        self.last_sent = Some((now, state, link));
        Some(self.json(now, wall_ms, clock_synced, state, link))
    }

    /// Final message on graceful shutdown.
    pub fn shutdown(&mut self, now: Instant, wall_ms: u64, clock_synced: bool) -> Value {
        self.json(now, wall_ms, clock_synced, "down", "disconnected")
    }

    fn json(
        &mut self,
        now: Instant,
        wall_ms: u64,
        clock_synced: bool,
        state: &str,
        link: &str,
    ) -> Value {
        while self
            .published
            .front()
            .is_some_and(|t| now.duration_since(*t) > RATE_WINDOW)
        {
            self.published.pop_front();
        }
        self.channels.retain(|_, t| now.duration_since(*t) <= FRESH);
        let mut channels: Vec<&String> = self.channels.keys().collect();
        channels.sort();
        let rate = (self.published.len() as f64 / RATE_WINDOW.as_secs_f64() * 10.0).round() / 10.0;
        json!({
            "schema": 1,
            "bridge": "can-to-mqtt",
            "version": env!("CARGO_PKG_VERSION"),
            "bootId": self.boot_id,
            "timestampMs": wall_ms,
            "state": state,
            "link": link,
            "sources": ["obd2/mode01"],
            "rateHz": rate,
            "ageMs": self.last_capture_ms.map(|t| wall_ms.saturating_sub(t)),
            "channels": channels,
            "errors": self.errors,
            "clockSynced": clock_synced,
        })
    }
}

/// Kernel clock discipline state: synced when adjtimex reports no TIME_ERROR
/// and STA_UNSYNC is clear (set by chrony/ntpd/timesyncd once locked).
#[cfg(target_os = "linux")]
pub fn clock_synced() -> bool {
    let mut tx: libc::timex = unsafe { std::mem::zeroed() };
    // SAFETY: modes = 0 only reads the kernel state into `tx`.
    let state = unsafe { libc::adjtimex(&mut tx) };
    state != libc::TIME_ERROR && tx.status & libc::STA_UNSYNC == 0
}

#[cfg(not(target_os = "linux"))]
pub fn clock_synced() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(ms: u64, key: &str) -> Value {
        json!({"schema": 1, "timestampMs": ms, "channels": {key: 1.0}})
    }

    #[test]
    fn state_machine_follows_link_freshness_and_errors() {
        let t0 = Instant::now();
        let mut h = Health::new("boot");
        assert_eq!((h.state(t0), h.link(t0)), ("down", "disconnected"));
        h.set_socket(true);
        assert_eq!((h.state(t0), h.link(t0)), ("down", "connecting"));
        h.on_reply(t0);
        // ECU answers but nothing publishable yet.
        assert_eq!((h.state(t0), h.link(t0)), ("degraded", "connected"));
        h.on_packet(&packet(1_000, "rpm"), t0);
        assert_eq!(h.state(t0), "ok");
        // Timeouts alone (unsupported PIDs) do not degrade.
        h.on_error("timeouts", t0);
        assert_eq!(h.state(t0), "ok");
        h.on_error("canErrors", t0);
        assert_eq!(h.state(t0), "degraded");
        assert_eq!(h.state(t0 + Duration::from_secs(5)), "degraded"); // packet now stale (> 2 s)
        let t1 = t0 + Duration::from_secs(6);
        h.on_reply(t1);
        h.on_packet(&packet(7_000, "rpm"), t1);
        assert_eq!(h.state(t1), "ok");
        assert_eq!(h.state(t1 + Duration::from_secs(11)), "down");
        h.set_socket(false);
        assert_eq!(h.link(t1), "disconnected");
    }

    #[test]
    fn json_shape_rate_and_publish_cadence() {
        let t0 = Instant::now();
        let mut h = Health::new("boot-1");
        h.set_socket(true);
        let first = h.due(t0, 10_000, true).expect("first health is due");
        assert_eq!(first["ageMs"], Value::Null);
        assert!(h.due(t0 + Duration::from_secs(1), 11_000, true).is_none());
        // 10 packets in the last 5 s -> 2.0 Hz; a link change forces a publish.
        for i in 0..10 {
            let t = t0 + Duration::from_millis(1_500 + i * 300);
            h.on_reply(t);
            h.on_packet(
                &packet(
                    11_000 + i * 300,
                    if i % 2 == 0 { "rpm" } else { "throttle" },
                ),
                t,
            );
        }
        let now = t0 + Duration::from_millis(4_500);
        let v = h
            .due(now, 13_720, true)
            .expect("link change publishes at once");
        assert_eq!(v["schema"], 1);
        assert_eq!(v["bridge"], "can-to-mqtt");
        assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(v["bootId"], "boot-1");
        assert_eq!(v["timestampMs"], 13_720);
        assert_eq!(v["state"], "ok");
        assert_eq!(v["link"], "connected");
        assert_eq!(v["sources"], json!(["obd2/mode01"]));
        assert_eq!(v["rateHz"], 2.0);
        assert_eq!(v["ageMs"], 13_720 - 13_700);
        assert_eq!(v["channels"], json!(["rpm", "throttle"]));
        assert_eq!(
            v["errors"],
            json!({"canErrors": 0, "publishErrors": 0, "timeouts": 0})
        );
        assert_eq!(v["clockSynced"], true);
        // Unchanged state: next publish after 5 s.
        assert!(
            h.due(now + Duration::from_millis(100), 13_820, true)
                .is_none()
        );
        let later = h.due(now + Duration::from_secs(5), 18_720, false).unwrap();
        assert_eq!(later["clockSynced"], false);
        assert_eq!(later["channels"], json!([])); // nothing fresh
        assert_eq!(h.shutdown(now, 18_800, true)["state"], "down");
    }
}

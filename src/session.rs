//! Pure bounded OBD discovery/poll planner. Only Mode 01 requests are produced.
use crate::{
    config::AppConfig,
    telemetry::{PIDS, positive_payload},
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::{Duration, Instant};
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct Identity {
    pub id: u32,
    pub extended: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub id: u32,
    pub extended: bool,
    pub pid: u8,
}
impl Request {
    pub fn bytes(&self) -> [u8; 8] {
        [2, 1, self.pid, 0, 0, 0, 0, 0]
    }
}
pub struct Session {
    config: AppConfig,
    pub selected: Option<Identity>,
    pub supported: BTreeSet<u8>,
    pub ready: bool,
    candidates: BTreeMap<Identity, [u8; 4]>,
    pending: HashMap<u8, Instant>,
    page: u8,
    next_probe: Instant,
    probe_extended: bool,
    high: Vec<u8>,
    regular: Vec<u8>,
    high_index: usize,
    regular_index: usize,
    next_regular: Instant,
}
impl Session {
    pub fn new(config: &AppConfig, now: Instant) -> Self {
        Self {
            config: config.clone(),
            selected: None,
            supported: BTreeSet::new(),
            ready: false,
            candidates: BTreeMap::new(),
            pending: HashMap::new(),
            page: 0,
            next_probe: now,
            probe_extended: false,
            high: vec![],
            regular: vec![],
            high_index: 0,
            regular_index: 0,
            next_regular: now,
        }
    }
    fn request(&self, pid: u8, extended: bool) -> Request {
        Request {
            id: match self.selected {
                Some(Identity {
                    id,
                    extended: false,
                }) if (0x7E8..=0x7EF).contains(&id) => id - 8,
                Some(Identity { id, extended: true }) if id & 0x1FFFFF00 == 0x18DAF100 => {
                    0x18DA00F1 | ((id & 0xFF) << 8)
                }
                _ => {
                    if extended {
                        0x18DB33F1
                    } else {
                        0x7DF
                    }
                }
            },
            extended,
            pid,
        }
    }
    fn bind(&mut self, id: Identity, map: [u8; 4], now: Instant) {
        self.selected = Some(id);
        self.pending.clear();
        self.apply_page(0, map);
        self.next_probe = now;
    }
    fn apply_page(&mut self, page: u8, map: [u8; 4]) {
        for i in 0..32u16 {
            if map[usize::from(i / 8)] & (1 << (7 - i % 8)) != 0 {
                let value = u16::from(page) + i + 1;
                if value <= 255 {
                    self.supported.insert(value as u8);
                }
            }
        }
        if page < 0xE0 && map[3] & 1 != 0 {
            self.page = page + 0x20
        } else {
            self.ready = true;
            let wanted = |p: &u8| {
                self.supported.contains(p)
                    && (self.config.enabled_pids.is_empty() || self.config.enabled_pids.contains(p))
            };
            self.high = [0x0C, 0x0D, 0x11, 0x04, 0x0E]
                .into_iter()
                .filter(wanted)
                .collect();
            self.regular = PIDS
                .iter()
                .copied()
                .filter(|p| !self.high.contains(p))
                .filter(wanted)
                .collect();
        }
    }
    pub fn requests(&mut self, now: Instant) -> Vec<Request> {
        if self.config.mode == "passive" {
            return vec![];
        }
        if self.selected.is_none() {
            if let Some(sent) = self.pending.get(&0)
                && now.saturating_duration_since(*sent) >= Duration::from_millis(150)
                && let Some((&id, &map)) = self.candidates.first_key_value()
            {
                self.bind(id, map, now)
            }
            if self.selected.is_none() {
                if now < self.next_probe {
                    return vec![];
                };
                self.pending.insert(0, now);
                self.next_probe = now + Duration::from_millis(500);
                let extended = match self.config.can_format.as_str() {
                    "extended" => true,
                    "standard" => false,
                    _ => {
                        let choice = self.probe_extended;
                        self.probe_extended = !choice;
                        choice
                    }
                };
                return vec![self.request(0, extended)];
            }
        }
        let extended = self.selected.is_some_and(|id| id.extended);
        if !self.ready {
            if now < self.next_probe {
                return vec![];
            };
            self.pending.insert(self.page, now);
            self.next_probe = now + Duration::from_millis(500);
            return vec![self.request(self.page, extended)];
        }
        self.pending
            .retain(|_, sent| now.saturating_duration_since(*sent) < Duration::from_millis(500));
        let pid = if !self.regular.is_empty() && (now >= self.next_regular || self.high.is_empty())
        {
            self.next_regular = now + Duration::from_millis(self.config.poll_interval_regular_ms);
            let p = self.regular[self.regular_index % self.regular.len()];
            self.regular_index += 1;
            p
        } else if !self.high.is_empty() {
            let p = self.high[self.high_index % self.high.len()];
            self.high_index += 1;
            p
        } else {
            return vec![];
        };
        if self.pending.contains_key(&pid) {
            return vec![];
        };
        self.pending.insert(pid, now);
        vec![self.request(pid, extended)]
    }
    pub fn observe(&mut self, id: Identity, bytes: &[u8], now: Instant) -> bool {
        if self.config.mode == "passive" {
            return false;
        }
        let response_id = id.id;
        let valid = if id.extended {
            response_id & 0x1FFFFF00 == 0x18DAF100
        } else {
            (0x7E8..=0x7EF).contains(&response_id)
        };
        if (!valid && self.config.obd_response_id != Some(response_id))
            || self
                .config
                .obd_response_id
                .is_some_and(|expected| expected != response_id)
        {
            return false;
        }
        if self.config.can_format == "standard" && id.extended
            || self.config.can_format == "extended" && !id.extended
        {
            return false;
        }
        let Some(payload) = positive_payload(bytes) else {
            return false;
        };
        let Some(&pid) = payload.get(1) else {
            return false;
        };
        if !self
            .pending
            .get(&pid)
            .is_some_and(|sent| now.saturating_duration_since(*sent) <= Duration::from_millis(500))
        {
            return false;
        }
        if pid % 0x20 == 0 {
            if payload.len() < 6 {
                return false;
            };
            let map = [payload[2], payload[3], payload[4], payload[5]];
            if self.selected.is_none() {
                if pid == 0 {
                    self.candidates.insert(id, map);
                };
                return false;
            }
            if self.selected != Some(id) || pid != self.page {
                return false;
            };
            self.pending.remove(&pid);
            self.apply_page(pid, map);
            return false;
        }
        if self.selected != Some(id) || !self.supported.contains(&pid) {
            return false;
        };
        self.pending.remove(&pid);
        PIDS.contains(&pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> AppConfig {
        AppConfig {
            car: Some("qa-car".into()),
            ..AppConfig::default()
        }
    }
    #[test]
    fn discovers_only_supported_pages_and_binds_one_ecu() {
        let now = Instant::now();
        let mut s = Session::new(&base(), now);
        assert_eq!(s.requests(now)[0].bytes(), [2, 1, 0, 0, 0, 0, 0, 0]);
        s.observe(
            Identity {
                id: 0x7E9,
                extended: false,
            },
            &[6, 0x41, 0, 0, 0x10, 0, 1],
            now + Duration::from_millis(20),
        );
        s.observe(
            Identity {
                id: 0x7E8,
                extended: false,
            },
            &[6, 0x41, 0, 0, 0x10, 0, 1],
            now + Duration::from_millis(30),
        );
        let r = s.requests(now + Duration::from_millis(160));
        assert_eq!(
            s.selected,
            Some(Identity {
                id: 0x7E8,
                extended: false
            })
        );
        assert_eq!(r[0].pid, 0x20);
        assert!(!s.observe(
            Identity {
                id: 0x7E9,
                extended: false
            },
            &[6, 0x41, 0x20, 0xFF, 0xFF, 0xFF, 0xFF],
            now + Duration::from_millis(180)
        ));
        assert!(!s.ready);
        s.observe(
            Identity {
                id: 0x7E8,
                extended: false,
            },
            &[6, 0x41, 0x20, 0, 0, 0, 1],
            now + Duration::from_millis(180),
        );
        assert_eq!(s.requests(now + Duration::from_millis(700))[0].pid, 0x40);
        s.observe(
            Identity {
                id: 0x7E8,
                extended: false,
            },
            &[6, 0x41, 0x40, 0x40, 0, 0, 0],
            now + Duration::from_millis(720),
        );
        assert!(s.ready);
        for i in 1..20 {
            for r in s.requests(now + Duration::from_millis(720 + i * 30)) {
                assert!([0x0C, 0x42].contains(&r.pid));
                assert_eq!(r.bytes()[1], 1);
                assert_eq!(r.id, 0x7E0)
            }
        }
    }
    #[test]
    fn auto_tries_extended_only_when_standard_does_not_answer() {
        let now = Instant::now();
        let mut s = Session::new(&base(), now);
        assert!(!s.requests(now)[0].extended);
        assert!(s.requests(now + Duration::from_millis(500))[0].extended);
        s.observe(
            Identity {
                id: 0x18DAF110,
                extended: true,
            },
            &[6, 0x41, 0, 0, 0x10, 0, 0],
            now + Duration::from_millis(520),
        );
        let r = s.requests(now + Duration::from_millis(660));
        assert!(r[0].extended);
        assert_eq!(r[0].id, 0x18DA10F1);
        assert_eq!(r[0].pid, 0x0C);
        assert!(s.observe(
            Identity {
                id: 0x18DAF110,
                extended: true
            },
            &[4, 0x41, 0x0C, 0x1F, 0x40],
            now + Duration::from_millis(670)
        ));
        assert!(!s.observe(
            Identity {
                id: 0x18DAF111,
                extended: true
            },
            &[4, 0x41, 0x0C, 0x1F, 0x40],
            now + Duration::from_millis(675)
        ));
    }
    #[test]
    fn passive_planner_never_creates_requests() {
        let now = Instant::now();
        let cfg = AppConfig {
            mode: "passive".into(),
            ..base()
        };
        let mut s = Session::new(&cfg, now);
        for i in 0..100 {
            assert!(s.requests(now + Duration::from_secs(i)).is_empty());
            assert!(!s.observe(
                Identity {
                    id: 0x7E8,
                    extended: false
                },
                &[6, 0x41, 0, 0xFF, 0xFF, 0xFF, 0xFF],
                now + Duration::from_secs(i)
            ));
        }
    }
    #[test]
    fn rejects_unsolicited_and_stale_responses() {
        let now = Instant::now();
        let mut s = Session::new(&base(), now);
        assert!(!s.observe(
            Identity {
                id: 0x7E8,
                extended: false
            },
            &[6, 0x41, 0, 0, 0x10, 0, 0],
            now
        ));
        s.requests(now);
        assert!(!s.observe(
            Identity {
                id: 0x7E8,
                extended: false
            },
            &[6, 0x41, 0, 0, 0x10, 0, 0],
            now + Duration::from_secs(2)
        ));
        assert!(s.selected.is_none());
    }
}

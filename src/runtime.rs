use crate::{config::AppConfig, state::Shared};
use serde_json::Value;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
#[derive(Clone)]
pub struct Outgoing {
    pub topic: String,
    pub packet: Value,
    pub at: Instant,
    pub retained: bool,
}
pub fn enqueue(
    tx: &mpsc::Sender<Outgoing>,
    shared: &Shared,
    suffix: &str,
    packet: Value,
    retained: bool,
) {
    if tx
        .try_send(Outgoing {
            topic: format!("{}/{}", shared.config.base(), suffix),
            packet,
            at: Instant::now(),
            retained,
        })
        .is_err()
    {
        shared.drop_packet()
    }
}
pub async fn run(config: AppConfig, web_only: bool) -> Result<(), crate::error::AppError> {
    let shared =
        Arc::new(Shared::new(config.clone()).map_err(crate::error::AppError::Initialization)?);
    if config.prometheus_enabled {
        crate::metrics::start_metrics_server(config.prometheus_port, shared.metrics.clone())
            .await
            .map_err(|error| crate::error::AppError::Initialization(error.to_string()))?;
    }
    let web = if config.http_enabled {
        Some(
            crate::web::start(shared.clone())
                .await
                .map_err(crate::error::AppError::Initialization)?,
        )
    } else {
        None
    };
    let (tx, mut rx) = mpsc::channel::<Outgoing>(64);
    let publisher_shared = shared.clone();
    let publisher = tokio::task::spawn_blocking(move || {
        let mut client: Option<paho_mqtt::Client> = None;
        let mut retry_at = Instant::now();
        let mut backoff = Duration::from_secs(1);
        while let Some(item) = rx.blocking_recv() {
            if item.at.elapsed() > Duration::from_millis(1500) {
                publisher_shared.drop_packet();
                continue;
            }
            if client.as_ref().is_none_or(|c| !c.is_connected()) {
                publisher_shared.mqtt(false);
                if Instant::now() < retry_at {
                    publisher_shared.drop_packet();
                    continue;
                }
                match crate::mqtt_handler::setup_mqtt(&publisher_shared.config) {
                    Ok(c) => {
                        client = Some(c);
                        publisher_shared.mqtt(true);
                        backoff = Duration::from_secs(1)
                    }
                    Err(_) => {
                        retry_at = Instant::now() + backoff;
                        backoff = (backoff * 2).min(Duration::from_secs(30));
                        publisher_shared.drop_packet();
                        continue;
                    }
                }
            }
            if item.at.elapsed() > Duration::from_millis(1500) {
                publisher_shared.drop_packet();
                continue;
            }
            if let Some(c) = &client {
                let msg = paho_mqtt::Message::new_retained(&item.topic, item.packet.to_string(), 0);
                let msg = if item.retained {
                    msg
                } else {
                    paho_mqtt::Message::new(&item.topic, item.packet.to_string(), 0)
                };
                if c.publish(msg).is_ok() {
                    if publisher_shared.config.legacy_scalar_output && !item.retained {
                        crate::mqtt_handler::publish_fresh_legacy(
                            c,
                            &publisher_shared.config,
                            &item.packet,
                        );
                    }
                    publisher_shared.published();
                    publisher_shared.mqtt(true)
                } else {
                    publisher_shared.mqtt(false);
                    publisher_shared.drop_packet()
                }
            }
        }
        if let Some(c) = client {
            let _ = c.publish(paho_mqtt::Message::new_retained(
                format!("{}/health", publisher_shared.config.base()),
                publisher_shared.health(true).to_string(),
                0,
            ));
            crate::mqtt_handler::publish_status(&c, &publisher_shared.config, false);
            let _ = c.disconnect(None);
        }
    });
    let acquisition = if web_only {
        shared.can(false, "Web preview only; acquisition is stopped");
        None
    } else {
        Some(tokio::spawn(acquire(config, shared.clone(), tx.clone())))
    };
    let status_shared = shared.clone();
    let status_tx = tx.clone();
    let health = tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        loop {
            tick.tick().await;
            let _ = sd_notify::notify(false, &[sd_notify::NotifyState::Watchdog]);
            enqueue(
                &status_tx,
                &status_shared,
                "health",
                status_shared.health(false),
                true,
            )
        }
    });
    shutdown().await;
    if let Some(t) = acquisition {
        t.abort();
        let _ = t.await;
    }
    health.abort();
    let _ = health.await;
    drop(tx);
    if let Some(web) = web {
        web.abort();
    }
    let _ = tokio::time::timeout(Duration::from_secs(12), publisher).await;
    Ok(())
}
async fn shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut term) = signal(SignalKind::terminate()) {
            tokio::select! {_=tokio::signal::ctrl_c()=>{},_=term.recv()=>{}};
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}
#[cfg(not(target_os = "linux"))]
async fn acquire(_: AppConfig, shared: Arc<Shared>, _: mpsc::Sender<Outgoing>) {
    shared.can(false,"SocketCAN requires Linux. Use a separate ELM adapter service for Bluetooth, serial or TCP transports on this platform");
}
#[cfg(target_os = "linux")]
async fn acquire(config: AppConfig, shared: Arc<Shared>, tx: mpsc::Sender<Outgoing>) {
    use crate::session::{Identity, Session};
    use futures_util::StreamExt;
    use socketcan::{
        CanFrame,
        embedded_can::{ExtendedId, Frame, Id, StandardId},
        tokio::CanSocket,
    };
    let mut sequence = 0u64;
    loop {
        let mut rx = match CanSocket::open(&config.can_interface) {
            Ok(s) => s,
            Err(_) => {
                shared.can(false,"CAN interface unavailable. Connect it, set the verified bitrate, then bring the interface up");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        // Passive mode does not even open a transmitter socket.
        let sender = if config.mode == "passive" {
            None
        } else {
            match CanSocket::open(&config.can_interface) {
                Ok(s) => Some(s),
                Err(_) => {
                    shared.can(false, "Could not open CAN request socket");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
        };
        shared.can(true, "");
        let mut session = Session::new(&config, Instant::now());
        let mut interval =
            tokio::time::interval(Duration::from_millis(config.poll_interval_high_freq_ms));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        'receive: loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Some(socket) = &sender {
                        for request in session.requests(Instant::now()) {
                            let id = if request.extended {
                                ExtendedId::new(request.id).map(Id::Extended)
                            } else {
                                StandardId::new(request.id as u16).map(Id::Standard)
                            };
                            if let Some(frame) = id.and_then(|id| CanFrame::new(id, &request.bytes())) {
                                if !matches!(tokio::time::timeout(Duration::from_millis(100), socket.write_frame(frame)).await, Ok(Ok(()))) {
                                    shared.can(false, "CAN request failed; check interface state");
                                    break 'receive;
                                }
                            }
                        }
                        shared.discovery(serde_json::to_value(session.selected).unwrap_or(Value::Null), session.supported.iter().copied().collect());
                    }
                },
                frame = rx.next() => {
                    match frame {
                        Some(Ok(CanFrame::Data(frame))) => {
                            shared.frame();
                            let (id, extended) = match frame.id() {
                                Id::Standard(id) => (u32::from(id.as_raw()), false),
                                Id::Extended(id) => (id.as_raw(), true),
                            };
                            let captured = std::time::SystemTime::now();
                            let now = Instant::now();
                            let identity = Identity { id, extended };
                            let packet = if config.mode == "passive" {
                                if config.passive_profile == "obd_mode01" {
                                    let format_matches = match config.can_format.as_str() {
                                        "standard" => !extended,
                                        "extended" => extended,
                                        _ => true,
                                    };
                                    if config.obd_response_id == Some(id) && format_matches {
                                        crate::telemetry::packet(frame.data(), &shared.boot, sequence + 1, captured).map(|mut packet| {
                                            packet["source"] = serde_json::json!("sensors/can/obd_mode01");
                                            packet
                                        })
                                    } else { None }
                                } else {
                                    crate::passive::packet(&config.passive_signals, &config.passive_profile, identity, frame.data(), &shared.boot, sequence + 1, crate::state::wall_ms())
                                }
                            } else if session.observe(identity, frame.data(), now) {
                                crate::telemetry::packet(frame.data(), &shared.boot, sequence + 1, captured)
                            } else { None };
                            if let Some(packet) = packet {
                                sequence += 1;
                                if shared.sample(&packet, now, crate::state::wall_ms()) {
                                    enqueue(&tx, &shared, "telemetry", packet, false);
                                }
                            }
                        },
                        Some(Ok(CanFrame::Error(_))) => {
                            shared.can(false, "CAN bus error; check wiring, termination and bitrate");
                            break;
                        },
                        Some(Ok(_)) => {},
                        Some(Err(_)) | None => {
                            shared.can(false, "CAN receive link lost");
                            break;
                        },
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[cfg(all(test, target_os = "linux"))]
mod vcan_tests {
    use super::*;
    use crate::passive::PassiveSignal;
    use futures_util::StreamExt;
    use socketcan::{
        CanFrame,
        embedded_can::{ExtendedId, Frame, Id, StandardId},
        tokio::CanSocket,
    };

    fn interface() -> String {
        std::env::var("G86_CAN_TEST_IFACE")
            .expect("Set G86_CAN_TEST_IFACE to an isolated, up virtual CAN interface")
    }
    fn frame(id: u32, extended: bool, bytes: &[u8]) -> CanFrame {
        let id = if extended {
            Id::Extended(ExtendedId::new(id).unwrap())
        } else {
            Id::Standard(StandardId::new(id as u16).unwrap())
        };
        CanFrame::new(id, bytes).unwrap()
    }
    async fn wait_open(shared: &Shared) {
        tokio::time::timeout(Duration::from_secs(3), async {
            while shared.snapshot(Instant::now())["can_connected"] != true {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
    }
    #[tokio::test]
    #[ignore = "Requires an isolated Linux virtual CAN interface; see docs/TESTING.md"]
    async fn passive_receives_real_frames_without_transmitting_or_redating() {
        let interface = interface();
        let mut spy = CanSocket::open(&interface).unwrap();
        let injector = CanSocket::open(&interface).unwrap();
        let mut config = AppConfig {
            can_interface: interface,
            car: Some("vcan-check".into()),
            mode: "passive".into(),
            passive_profile: "verified".into(),
            passive_signals: vec![PassiveSignal {
                channel: "rpm".into(),
                can_id: 0x123,
                bit_length: 16,
                source_reference: "Isolated virtual CAN fixture: unsigned little-endian rpm".into(),
                verified: true,
                ..PassiveSignal::default()
            }],
            ..AppConfig::default()
        };
        config.resolve_base_topic();
        config.validate().unwrap();
        let shared = Arc::new(Shared::new(config.clone()).unwrap());
        let (tx, mut rx) = mpsc::channel(64);
        let task = tokio::spawn(acquire(config, shared.clone(), tx));
        wait_open(&shared).await;
        injector
            .write_frame(frame(0x123, false, &[0xD0, 0x07]))
            .await
            .unwrap();
        let packet = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(packet.topic, "/g86/vcan-check/sensors/verified/telemetry");
        assert_eq!(packet.packet["channels"], serde_json::json!({"rpm":2000.0}));
        assert_eq!(packet.packet["source"], "sensors/can/verified");
        assert!(!packet.retained);
        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(1), spy.next()).await,
            Ok(Some(Ok(_)))
        ));
        assert!(
            tokio::time::timeout(Duration::from_millis(350), spy.next())
                .await
                .is_err(),
            "Passive service transmitted a CAN frame"
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(100), rx.recv())
                .await
                .is_err(),
            "Cached sample was republished"
        );
        assert_eq!(
            shared.snapshot(Instant::now() + Duration::from_millis(2100))["channels"],
            serde_json::json!([])
        );
        task.abort();
        let _ = task.await;
    }
    #[tokio::test]
    #[ignore = "Requires an isolated Linux virtual CAN interface; see docs/TESTING.md"]
    async fn discovers_and_polls_real_standard_and_extended_can_frames() {
        for extended in [false, true] {
            let interface = interface();
            let mut ecu = CanSocket::open(&interface).unwrap();
            let mut config = AppConfig {
                can_interface: interface,
                car: Some("vcan-check".into()),
                can_format: if extended {
                    "extended".into()
                } else {
                    "standard".into()
                },
                ..AppConfig::default()
            };
            config.resolve_base_topic();
            let shared = Arc::new(Shared::new(config.clone()).unwrap());
            let (tx, mut rx) = mpsc::channel(64);
            let task = tokio::spawn(acquire(config, shared.clone(), tx));
            wait_open(&shared).await;
            let response_id = if extended { 0x18DAF110 } else { 0x7E8 };
            let physical_id = if extended { 0x18DA10F1 } else { 0x7E0 };
            let mut bound = false;
            tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    let request = ecu.next().await.unwrap().unwrap();
                    let CanFrame::Data(request) = request else {
                        continue;
                    };
                    let bytes = request.data();
                    assert_eq!(
                        &bytes[..2],
                        &[2, 1],
                        "Only Mode 01 reads may be transmitted"
                    );
                    let response = match bytes[2] {
                        0 => vec![6, 0x41, 0, 0, 0x10, 0, 0, 0],
                        0x0C => {
                            let actual = match request.id() {
                                Id::Standard(id) => u32::from(id.as_raw()),
                                Id::Extended(id) => id.as_raw(),
                            };
                            assert_eq!(actual, physical_id);
                            bound = true;
                            vec![4, 0x41, 0x0C, 0x1F, 0x40, 0, 0, 0]
                        }
                        unexpected => panic!("Polled unsupported PID {unexpected:X}"),
                    };
                    ecu.write_frame(frame(response_id, extended, &response))
                        .await
                        .unwrap();
                    if bound {
                        break;
                    }
                }
            })
            .await
            .unwrap();
            let packet = tokio::time::timeout(Duration::from_secs(2), rx.recv())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(packet.topic, "/g86/vcan-check/obd/telemetry");
            assert_eq!(packet.packet["channels"], serde_json::json!({"rpm":2000.0}));
            assert_eq!(packet.packet["source"], "obd2/mode01");
            assert_eq!(packet.packet["sequence"], 1);
            assert!(
                packet.packet["timestampMs"]
                    .as_u64()
                    .unwrap()
                    .abs_diff(crate::state::wall_ms())
                    < 1000
            );
            task.abort();
            let _ = task.await;
        }
    }
}

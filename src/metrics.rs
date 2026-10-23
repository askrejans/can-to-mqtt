//! Prometheus metrics and HTTP health check endpoints.
//!
//! This module provides comprehensive metrics collection and export
//! for monitoring the CAN-to-MQTT bridge in production.

use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Health status for the application
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub can_connected: bool,
    pub mqtt_connected: bool,
    pub uptime_seconds: u64,
    pub last_can_frame_seconds_ago: Option<u64>,
    pub last_mqtt_publish_seconds_ago: Option<u64>,
    pub last_telemetry_seconds_ago: Option<u64>,
}

/// Shared health state
pub struct HealthState {
    pub start_time: SystemTime,
    pub can_connected: Arc<RwLock<bool>>,
    pub mqtt_connected: Arc<RwLock<bool>>,
    pub last_can_frame: Arc<RwLock<Option<SystemTime>>>,
    pub last_mqtt_publish: Arc<RwLock<Option<SystemTime>>>,
    pub last_telemetry: Arc<RwLock<Option<SystemTime>>>,
}

impl Default for HealthState {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthState {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            can_connected: Arc::new(RwLock::new(false)),
            mqtt_connected: Arc::new(RwLock::new(false)),
            last_can_frame: Arc::new(RwLock::new(None)),
            last_mqtt_publish: Arc::new(RwLock::new(None)),
            last_telemetry: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_health(&self) -> HealthStatus {
        let can_connected = *self.can_connected.read().await;
        let mqtt_connected = *self.mqtt_connected.read().await;
        let uptime = self.start_time.elapsed().unwrap_or_default().as_secs();

        let last_can_frame_seconds_ago = match *self.last_can_frame.read().await {
            Some(time) => time.elapsed().ok().map(|d| d.as_secs()),
            None => None,
        };

        let last_mqtt_publish_seconds_ago = match *self.last_mqtt_publish.read().await {
            Some(time) => time.elapsed().ok().map(|d| d.as_secs()),
            None => None,
        };

        let last_telemetry_seconds_ago = self
            .last_telemetry
            .read()
            .await
            .and_then(|at| at.elapsed().ok())
            .map(|age| age.as_secs());
        let telemetry_fresh = self.last_telemetry.read().await.is_some_and(|at| {
            at.elapsed()
                .is_ok_and(|age| age <= std::time::Duration::from_secs(2))
        });
        let status = if can_connected && mqtt_connected && telemetry_fresh {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        };

        HealthStatus {
            status,
            can_connected,
            mqtt_connected,
            uptime_seconds: uptime,
            last_can_frame_seconds_ago,
            last_mqtt_publish_seconds_ago,
            last_telemetry_seconds_ago,
        }
    }
}

/// Initialize Prometheus metrics registry
pub fn init_metrics() {
    describe_counter!("can_frames_sent_total", "Total number of CAN frames sent");
    describe_counter!(
        "can_frames_received_total",
        "Total number of CAN frames received"
    );
    describe_counter!("can_errors_total", "Total number of CAN bus errors");
    describe_counter!(
        "mqtt_publishes_total",
        "Total number of MQTT messages published"
    );
    describe_counter!(
        "mqtt_publish_errors_total",
        "Total number of MQTT publish errors"
    );
    describe_counter!(
        "mqtt_reconnections_total",
        "Total number of MQTT reconnection attempts"
    );
    describe_counter!("obd_requests_total", "Total number of OBD-II requests sent");
    describe_counter!(
        "obd_responses_total",
        "Total number of OBD-II responses received"
    );
    describe_counter!(
        "obd_timeouts_total",
        "Total number of OBD-II request timeouts"
    );
    describe_histogram!(
        "obd_request_duration_seconds",
        "Duration of OBD-II request/response cycles"
    );
    describe_histogram!(
        "mqtt_publish_duration_seconds",
        "Duration of MQTT publish operations"
    );
    describe_gauge!(
        "can_connected",
        "CAN bus connection status (1=connected, 0=disconnected)"
    );
    describe_gauge!(
        "mqtt_connected",
        "MQTT broker connection status (1=connected, 0=disconnected)"
    );
}

/// Record CAN frame sent
pub fn record_can_frame_sent() {
    counter!("can_frames_sent_total").increment(1);
}

/// Record CAN frame received
pub fn record_can_frame_received() {
    counter!("can_frames_received_total").increment(1);
}

/// Record CAN error
pub fn record_can_error() {
    counter!("can_errors_total").increment(1);
}

/// Record MQTT publish
pub fn record_mqtt_publish() {
    counter!("mqtt_publishes_total").increment(1);
}

/// Record MQTT publish error
pub fn record_mqtt_publish_error() {
    counter!("mqtt_publish_errors_total").increment(1);
}

/// Record MQTT reconnection attempt
pub fn record_mqtt_reconnection() {
    counter!("mqtt_reconnections_total").increment(1);
}

/// Record OBD request
pub fn record_obd_request() {
    counter!("obd_requests_total").increment(1);
}

/// Record OBD response
pub fn record_obd_response() {
    counter!("obd_responses_total").increment(1);
}

/// Record OBD timeout
pub fn record_obd_timeout() {
    counter!("obd_timeouts_total").increment(1);
}

/// Record OBD request duration
pub fn record_obd_duration(duration_secs: f64) {
    histogram!("obd_request_duration_seconds").record(duration_secs);
}

/// Record MQTT publish duration
pub fn record_mqtt_duration(duration_secs: f64) {
    histogram!("mqtt_publish_duration_seconds").record(duration_secs);
}

/// Update CAN connection status
pub fn set_can_connected(connected: bool) {
    gauge!("can_connected").set(if connected { 1.0 } else { 0.0 });
}

/// Update MQTT connection status
pub fn set_mqtt_connected(connected: bool) {
    gauge!("mqtt_connected").set(if connected { 1.0 } else { 0.0 });
}

/// Start Prometheus metrics exporter and HTTP server
pub async fn start_metrics_server(
    port: u16,
    health_state: Arc<HealthState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use http_body_util::Full;
    use hyper::body::Bytes;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Request, Response, StatusCode};
    use hyper_util::rt::TokioIo;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    // Install Prometheus recorder and get handle
    let builder = PrometheusBuilder::new();
    let handle = builder
        .install_recorder()
        .map_err(|e| format!("Failed to install Prometheus recorder: {}", e))?;

    init_metrics();

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    info!("Metrics server listening on http://{}", addr);

    tokio::spawn(async move {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            let io = TokioIo::new(stream);
            let health_state = health_state.clone();
            let handle = handle.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                    let health_state = health_state.clone();
                    let handle = handle.clone();
                    async move {
                        match req.uri().path() {
                            "/metrics" => {
                                let metrics = handle.render();

                                Ok::<_, hyper::Error>(
                                    Response::builder()
                                        .status(StatusCode::OK)
                                        .header("Content-Type", "text/plain; version=0.0.4")
                                        .body(Full::new(Bytes::from(metrics)))
                                        .unwrap(),
                                )
                            }
                            "/health" => {
                                let health = health_state.get_health().await;
                                let status = if health.status == "healthy" {
                                    StatusCode::OK
                                } else {
                                    StatusCode::SERVICE_UNAVAILABLE
                                };

                                let body = serde_json::to_string(&health).unwrap_or_default();

                                Ok::<_, hyper::Error>(
                                    Response::builder()
                                        .status(status)
                                        .header("Content-Type", "application/json")
                                        .body(Full::new(Bytes::from(body)))
                                        .unwrap(),
                                )
                            }
                            "/ready" => {
                                let health = health_state.get_health().await;
                                let status = if health.status == "healthy" {
                                    StatusCode::OK
                                } else {
                                    StatusCode::SERVICE_UNAVAILABLE
                                };

                                Ok::<_, hyper::Error>(
                                    Response::builder()
                                        .status(status)
                                        .body(Full::new(Bytes::from("ready")))
                                        .unwrap(),
                                )
                            }
                            "/live" => Ok::<_, hyper::Error>(
                                Response::builder()
                                    .status(StatusCode::OK)
                                    .body(Full::new(Bytes::from("alive")))
                                    .unwrap(),
                            ),
                            _ => Ok::<_, hyper::Error>(
                                Response::builder()
                                    .status(StatusCode::NOT_FOUND)
                                    .body(Full::new(Bytes::from("Not Found")))
                                    .unwrap(),
                            ),
                        }
                    }
                });

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    error!("Error serving connection: {:?}", err);
                }
            });
        }
    });

    Ok(())
}

//! Loopback-only authenticated setup and telemetry. Saving creates a private
//! validated draft; it cannot restart acquisition or send vehicle commands.
use crate::{config::AppConfig, state::Shared};
use http_body_util::{BodyExt, Full, Limited};
use hyper::{
    Request, Response, StatusCode,
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use serde_json::{Value, json};
use std::{
    convert::Infallible,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;
use tokio::{net::TcpListener, task::JoinHandle};
const INDEX: &str = include_str!("../web/index.html");
const CSS: &str = include_str!("../web/app.css");
const JS: &str = include_str!("../web/app.js");
const CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; font-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";
fn response(status: StatusCode, body: impl Into<Bytes>, kind: &str) -> Response<Full<Bytes>> {
    let mut r = Response::new(Full::new(body.into()));
    *r.status_mut() = status;
    for (k, v) in [
        ("content-type", kind),
        ("content-security-policy", CSP),
        ("cache-control", "no-store"),
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("referrer-policy", "no-referrer"),
    ] {
        if let Ok(value) = v.parse() {
            r.headers_mut()
                .insert(hyper::header::HeaderName::from_static(k), value);
        }
    }
    r
}
fn data(status: StatusCode, value: Value) -> Response<Full<Bytes>> {
    response(status, value.to_string(), "application/json; charset=utf-8")
}
fn auth(headers: &hyper::HeaderMap, token: &str) -> bool {
    headers
        .get(hyper::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|provided| provided.as_bytes().ct_eq(token.as_bytes()).into())
}
fn safe_host(headers: &hyper::HeaderMap) -> bool {
    headers
        .get(hyper::header::HOST)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<hyper::http::uri::Authority>().ok())
        .is_some_and(|a| {
            a.host() == "localhost"
                || a.host()
                    .trim_matches(['[', ']'])
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
        })
}
fn mutation_allowed(headers: &hyper::HeaderMap) -> bool {
    if headers.get("x-g86-action").and_then(|v| v.to_str().ok()) != Some("configure") {
        return false;
    }
    let origin = headers.get("origin").and_then(|v| v.to_str().ok());
    let host = headers.get("host").and_then(|v| v.to_str().ok());
    origin.is_none() || host.is_some_and(|h| origin == Some(format!("http://{h}").as_str()))
}
pub fn private_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)
        .map_err(|_| "Could not create the private settings directory")?;
    let mut nonce = [0u8; 8];
    getrandom::fill(&mut nonce).map_err(|_| "Could not create a private file identity")?;
    let suffix: String = nonce.iter().map(|b| format!("{b:02x}")).collect();
    let temp = parent.join(format!(".g86-can-{suffix}.tmp"));
    let result = (|| {
        use std::io::Write;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&temp)
            .map_err(|_| "Could not create the private settings file")?;
        file.write_all(bytes)
            .map_err(|_| "Could not save settings")?;
        file.sync_all().map_err(|_| "Could not flush settings")?;
        std::fs::rename(&temp, path).map_err(|_| "Could not replace the settings file")?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temp);
    };
    result
}
fn token(config: &AppConfig) -> Result<String, String> {
    if let Some(token) = &config.http_token {
        return Ok(token.clone());
    };
    let path = Path::new(&config.http_token_file);
    if path.exists() {
        let metadata =
            std::fs::metadata(path).map_err(|_| "Could not inspect the local access-token file")?;
        if metadata.len() > 256 {
            return Err("Invalid local access-token file".into());
        };
        let token = std::fs::read_to_string(path)
            .map_err(|_| "Could not read the local access-token file")?
            .trim()
            .to_string();
        if token.len() == 64 && token.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Ok(token);
        };
        return Err("Invalid local access-token file".into());
    };
    let mut random = [0u8; 32];
    getrandom::fill(&mut random).map_err(|_| "Could not create a local access token")?;
    let token: String = random.iter().map(|b| format!("{b:02x}")).collect();
    private_write(path, format!("{token}\n").as_bytes())?;
    tracing::info!(path=%path.display(),"Local UI access token saved privately; read this file to sign in");
    Ok(token)
}
pub fn configure(current: &AppConfig, input: Value) -> Result<AppConfig, String> {
    let obj = input.as_object().ok_or("Settings must be an object")?;
    let mut value = serde_json::to_value(current).map_err(|_| "Could not read settings")?;
    let target = value.as_object_mut().ok_or("Could not read settings")?;
    let fields = [
        "can_interface",
        "mode",
        "can_format",
        "obd_response_id",
        "passive_profile",
        "passive_signals",
        "car",
        "mqtt_host",
        "mqtt_port",
        "mqtt_username",
        "mqtt_tls",
        "mqtt_ca_file",
        "enabled_pids",
        "poll_interval_high_freq_ms",
        "poll_interval_regular_ms",
    ];
    for field in fields {
        if let Some(value) = obj.get(field) {
            target.insert(field.into(), value.clone());
        }
    }
    if let Some(password) = obj.get("mqtt_password").and_then(Value::as_str)
        && !password.is_empty()
        && password != "<redacted>"
    {
        target.insert("mqtt_password".into(), json!(password));
        target.insert("mqtt_password_file".into(), Value::Null);
    }
    let mut next: AppConfig =
        serde_json::from_value(value).map_err(|_| "Settings have an invalid type or range")?;
    next.mqtt_base_topic_explicit = current.mqtt_base_topic_explicit;
    if let Some(prefix) = obj.get("mqtt_base_topic").and_then(Value::as_str) {
        next.mqtt_base_topic_explicit = !prefix.is_empty();
        if !prefix.is_empty() {
            next.mqtt_base_topic = prefix.into()
        }
    }
    next.resolve_base_topic();
    next.validate().map_err(|e| e.to_string())?;
    Ok(next)
}
fn save_draft(config: &AppConfig) -> Result<(), String> {
    let mut value = serde_json::to_value(config).map_err(|_| "Could not encode settings")?;
    if let Some(obj) = value.as_object_mut() {
        if !config.mqtt_base_topic_explicit {
            obj.remove("mqtt_base_topic");
        }
        if config.mqtt_password_file.is_some() {
            obj.remove("mqtt_password");
        }
    }
    let bytes = serde_json::to_vec_pretty(&value).map_err(|_| "Could not encode settings")?;
    private_write(Path::new(&config.config_draft_path), &bytes)
}
pub fn interfaces() -> Value {
    let mut rows: Vec<Value> = vec![];
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                if std::fs::read_to_string(entry.path().join("type"))
                    .is_ok_and(|s| s.trim() == "280")
                {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let state =
                        std::fs::read_to_string(entry.path().join("operstate")).unwrap_or_default();
                    rows.push(json!({"name":name,"state":state.trim()}));
                }
            }
        }
    }
    rows.sort_by_key(|v| v["name"].as_str().unwrap_or("").to_string());
    json!({"platform":std::env::consts::OS,"interfaces":rows,"passive_changes_interface":false,"note":"Discovery reads interface metadata only. Set the verified bitrate with your system network tools."})
}
async fn handle(
    req: Request<Incoming>,
    shared: Arc<Shared>,
    token: Arc<String>,
) -> Response<Full<Bytes>> {
    if !safe_host(req.headers()) {
        return data(
            StatusCode::FORBIDDEN,
            json!({"error":"Use the loopback address shown by the service"}),
        );
    }
    let path = req.uri().path();
    if req.method() == hyper::Method::GET {
        match path {
            "/" => return response(StatusCode::OK, INDEX, "text/html; charset=utf-8"),
            "/app.css" => return response(StatusCode::OK, CSS, "text/css; charset=utf-8"),
            "/app.js" => return response(StatusCode::OK, JS, "text/javascript; charset=utf-8"),
            "/health" | "/healthz" => {
                let ready = shared.snapshot(Instant::now())["ready"] == true;
                return data(
                    if ready {
                        StatusCode::OK
                    } else {
                        StatusCode::SERVICE_UNAVAILABLE
                    },
                    json!({"ready":ready}),
                );
            }
            _ => {}
        }
    }
    if !auth(req.headers(), &token) {
        return data(
            StatusCode::UNAUTHORIZED,
            json!({"error":"Enter the local access token"}),
        );
    }
    if req.method() == hyper::Method::GET {
        let value = match path {
            "/api/v1/status" | "/api/v1/live" => shared.snapshot(Instant::now()),
            "/api/v1/config" => shared.config.redacted(),
            "/api/v1/discovery" => interfaces(),
            "/api/v1/channels" => json!({"mode01_pids":crate::telemetry::PIDS}),
            _ => return data(StatusCode::NOT_FOUND, json!({"error":"Route not found"})),
        };
        return data(StatusCode::OK, value);
    }
    if req.method() != hyper::Method::POST || path != "/api/v1/config" {
        return data(StatusCode::NOT_FOUND, json!({"error":"Route not found"}));
    }
    if !mutation_allowed(req.headers()) {
        return data(
            StatusCode::FORBIDDEN,
            json!({"error":"Use the setup screen to save a draft"}),
        );
    }
    let bytes = match Limited::new(req.into_body(), 64 * 1024).collect().await {
        Ok(body) => body.to_bytes(),
        Err(_) => {
            return data(
                StatusCode::PAYLOAD_TOO_LARGE,
                json!({"error":"Settings are too large"}),
            );
        }
    };
    let input = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => {
            return data(
                StatusCode::BAD_REQUEST,
                json!({"error":"Settings must be valid JSON"}),
            );
        }
    };
    match configure(&shared.config, input).and_then(|next| save_draft(&next).map(|_| next)) {
        Ok(next) => data(
            StatusCode::OK,
            json!({"saved":true,"requires_restart":true,"path":next.config_draft_path,"config":next.redacted(),"note":"Acquisition is unchanged. Stop the service, then restart with this draft."}),
        ),
        Err(error) => data(StatusCode::BAD_REQUEST, json!({"error":error})),
    }
}
pub async fn start(shared: Arc<Shared>) -> Result<JoinHandle<()>, String> {
    let token = Arc::new(token(&shared.config)?);
    let address = format!("{}:{}", shared.config.http_bind, shared.config.http_port);
    let listener = TcpListener::bind(&address)
        .await
        .map_err(|_| "Could not bind the local UI port")?;
    tracing::info!(address=%address,"CAN setup UI ready on loopback");
    let limit = Arc::new(tokio::sync::Semaphore::new(16));
    Ok(tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                continue;
            };
            let Ok(permit) = limit.clone().try_acquire_owned() else {
                continue;
            };
            let shared = shared.clone();
            let token = token.clone();
            tokio::spawn(async move {
                let _permit = permit;
                let service = service_fn(move |req| {
                    let shared = shared.clone();
                    let token = token.clone();
                    async move { Ok::<_, Infallible>(handle(req, shared, token).await) }
                });
                let _ = tokio::time::timeout(
                    Duration::from_secs(30),
                    http1::Builder::new().serve_connection(TokioIo::new(stream), service),
                )
                .await;
            });
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    fn headers(token: &str) -> hyper::HeaderMap {
        let mut h = hyper::HeaderMap::new();
        h.insert("host", "127.0.0.1:9093".parse().unwrap());
        h.insert("authorization", format!("Bearer {token}").parse().unwrap());
        h
    }
    #[test]
    fn bearer_origin_host_and_mutation_guards() {
        let token = "a".repeat(64);
        let mut h = headers(&token);
        assert!(auth(&h, &token));
        assert!(!auth(&h, &"b".repeat(64)));
        assert!(!mutation_allowed(&h));
        h.insert("x-g86-action", "configure".parse().unwrap());
        assert!(mutation_allowed(&h));
        h.insert("origin", "https://untrusted.invalid".parse().unwrap());
        assert!(!mutation_allowed(&h));
        h.insert("origin", "http://127.0.0.1:9093".parse().unwrap());
        assert!(mutation_allowed(&h));
        assert!(safe_host(&h));
        h.insert("host", "untrusted.invalid:9093".parse().unwrap());
        assert!(!safe_host(&h));
    }
    #[test]
    fn private_drafts_preserve_secrets_and_do_not_change_live_config() {
        let current = AppConfig {
            car: Some("car86".into()),
            mqtt_password: Some("test-secret-password".into()),
            ..AppConfig::default()
        };
        let changed=configure(&current,json!({"mode":"passive","mqtt_password":"","http_token":"not-allowed","http_bind":"0.0.0.0"})).unwrap();
        assert_eq!(current.mode, "obd");
        assert_eq!(changed.mode, "passive");
        assert_eq!(changed.mqtt_password, current.mqtt_password);
        assert_eq!(changed.http_token, current.http_token);
        assert_eq!(changed.http_bind, "127.0.0.1");
        assert_eq!(changed.base(), "/g86/car86/sensors/can");
        assert!(configure(&current, json!({"mqtt_port":-1})).is_err());
        assert!(configure(&current, json!({"can_interface":"can0; command"})).is_err());
    }
    #[test]
    fn assets_have_no_inline_script_external_resources_or_html_injection() {
        assert!(!INDEX.contains("<script>"));
        for text in [INDEX, CSS, JS] {
            assert!(!text.contains("https://") && !text.contains("http://"));
        }
        assert!(!JS.contains("innerHTML"));
        assert!(CSP.contains("frame-ancestors 'none'"));
    }
    #[tokio::test]
    async fn real_http_enforces_auth_and_saves_only_a_private_restart_draft() {
        let token = "a".repeat(64);
        let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();
        drop(socket);
        let dir = std::env::temp_dir().join(format!("g86-can-http-{}-{port}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let draft = dir.join("draft.json");
        let cfg = AppConfig {
            http_port: port,
            http_token: Some(token.clone()),
            mqtt_password: Some("private-broker-password".into()),
            config_draft_path: draft.to_string_lossy().into(),
            ..AppConfig::default()
        };
        let shared = Arc::new(Shared::new(cfg).unwrap());
        let server = start(shared.clone()).await.unwrap();
        async fn request(port: u16, path: &str, method: &str, extra: &str, body: &str) -> String {
            let mut socket = tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .unwrap();
            let value = format!(
                "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n{extra}Content-Length: {}\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(value.as_bytes()).await.unwrap();
            let mut result = vec![];
            socket.read_to_end(&mut result).await.unwrap();
            String::from_utf8(result).unwrap()
        }
        let no = request(port, "/api/v1/config", "GET", "", "").await;
        assert!(no.starts_with("HTTP/1.1 401"));
        assert!(!no.contains("private-broker-password"));
        let header = format!("Authorization: Bearer {token}\r\n");
        let config = request(port, "/api/v1/config", "GET", &header, "").await;
        assert!(config.starts_with("HTTP/1.1 200"));
        assert!(!config.contains("private-broker-password"));
        assert!(!config.contains(&token));
        assert!(config.contains("no-store"));
        let denied = request(port, "/api/v1/config", "POST", &header, "{}").await;
        assert!(denied.starts_with("HTTP/1.1 403"));
        assert!(!draft.exists());
        let accepted = request(
            port,
            "/api/v1/config",
            "POST",
            &format!("{header}X-G86-Action: configure\r\nContent-Type: application/json\r\n"),
            "{\"mode\":\"passive\",\"car\":\"qa-car\"}",
        )
        .await;
        assert!(accepted.starts_with("HTTP/1.1 200"));
        assert!(accepted.contains("requires_restart"));
        assert_eq!(shared.config.mode, "obd");
        assert!(draft.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&draft).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        let reloaded = crate::config::load_configuration(draft.to_str()).unwrap();
        assert_eq!(reloaded.mode, "passive");
        assert_eq!(reloaded.base(), "/g86/qa-car/sensors/can");
        server.abort();
        std::fs::remove_dir_all(dir).unwrap();
    }
}

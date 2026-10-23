# Changelog

## 0.5.0

- **Migration:** an unconfigured installation now publishes below `/telemetry/can` instead of `/GOLF86/ECU/`. Set `mqtt_base_topic` explicitly to `/GOLF86/ECU/` before upgrading if consumers need the former default. Explicit settings and optional `car` namespaces keep their topics.

- Document standalone use with arbitrary MQTT brokers and explicit topic prefixes; preserve explicit topic prefixes and telemetry packet contracts.
- Add an explicit MQTT topic prefix to guided setup; vehicle scope is optional. Accept `X-Bridge-Action` for configuration while preserving `X-G86-Action`.

## 0.4.0

- Add an embedded, authenticated Connect → MQTT → Live check interface with local dark/light assets and responsive layouts.
- Save validated owner-only configuration drafts with an explicit restart; redact credentials and keep the HTTP service on loopback.
- Discover standard Mode 01 supported PID pages, select one 11/29-bit ECU, physically address standard ECU mappings, and poll only advertised implemented measurements.
- Add verified passive signal mappings and passive observation of a bound ECU's existing OBD responses; passive mode opens no transmit socket.
- Emit newly acquired partial canonical packets with acquisition UTC, per-process identity, increasing sequence, exact phone-compatible sources, canonical ranges and bounded nonblocking live queues.
- Replace zero-filled scalar snapshots with optional actual fresh scalar migration output; preserve explicit legacy topic precedence.
- Add source-aware health/readiness and loopback metrics, real-process HTTP/browser checks, and isolated Linux virtual CAN checks.
- Update locked dependencies, remove the unmaintained atty dependency, add validated MQTT TLS/CA and private password-file support.
- License first-party source GPL-3.0-only, preserving third-party terms and earlier release grants.

The native acquisition path is Linux classical SocketCAN. ELM BLE/serial/TCP lives in the integrated G86 Car Computer service. No physical ECU, vehicle broadcast mapping or downstream phone/pit-wall acceptance is implied by virtual CAN tests.

## 0.3.1

### Added
- `car` setting (`CAN2MQTT_CAR`): base topic `/g86/<car>/obd` for the G86 car
  computer. An explicit `mqtt_base_topic` (now also `CAN2MQTT_MQTT_BASE_TOPIC`)
  still wins; with neither, the legacy `/GOLF86/ECU/` default is unchanged.
- `<base>/status` availability (`online`/`offline`, retained, MQTT last will).
- `<base>/health` retained bridge health JSON (state, link, rate, age of the
  last capture, fresh channels, error counters, clock sync), every 5 s and on
  a state change; a final `down` on graceful shutdown.

### Changed
- `car` and `mqtt_base_topic` are validated (slug; no wildcards).
- Clippy clean with `-D warnings`.

Earlier releases: see [RELEASE-0.3.0.md](RELEASE-0.3.0.md).

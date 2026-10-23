---
name: can-telemetry
description: Maintain can-to-mqtt SocketCAN acquisition, supported-PID discovery, verified passive CAN mappings, the local setup UI, and canonical MQTT release behavior. Use for protocol, freshness, configuration, dashboard, or release changes in this repository.
---

# CAN telemetry workflow

Read the relevant contracts before editing: `README.md`, `src/example.settings.toml`, and `docs/PASSIVE_SIGNALS.md`. For packaging read `docs/RELEASING.md`; for acceptance checks read `docs/TESTING.md`.

1. Define the expected real input, canonical values/units, timestamp, and missing-data behavior. Add a focused protocol regression for a bug or a new decoder.
2. Preserve the two acquisition boundaries. `session.rs` produces only standard Mode 01 reads and binds one ECU; `runtime.rs` passive mode opens only a receive socket. Passive mappings require an explicit verified reference.
3. Omit malformed, unsupported, out-of-range or stale data. Never initialize absent channels to zero or republish cached samples with a new acquisition time. Keep canonical source `obd2/mode01` for OBD and `sensors/can/<profile>` for passive mappings.
4. Keep embedded assets readable, local and free of inline script/HTML injection. Validate drafts before owner-only atomic writes; do not alter the running configuration or add hidden vehicle actions.
5. Run `cargo test --all-targets --locked`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo audit`. For UI changes run the real-process browser check. For acquisition changes run the ignored tests on a temporary isolated virtual CAN interface, then remove it.
6. Record exactly what was tested. A virtual CAN exchange proves the software path; it does not prove vehicle wiring, a physical ECU/gateway, Bluetooth/ELM hardware, the phone, or pit-wall delivery.

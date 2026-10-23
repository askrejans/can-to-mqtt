# can-to-mqtt 0.4.0

The standalone Linux bridge now has a guided local Connect → MQTT → Live check interface. Source data is bound to one ECU, discovered through standard supported-PID pages, and published only when newly acquired. The dashboard shows real channel ages, broker state and verified measurements instead of treating a saved setting as a successful connection.

Passive mode opens no transmit socket. Engineer-verified mappings provide canonical broadcast telemetry with explicit evidence, range checks and documented bit order. No speculative vehicle profiles or zero-filled missing channels are included.

Configuration drafts are authenticated, redacted in API responses and saved owner-only for explicit restart. The UI and metrics stay on loopback. MQTT TLS validates both certificate trust and hostname. Canonical telemetry uses nonretained QoS 0 and a bounded queue so broker failures cannot block acquisition or replay stale data as fresh.

Validation covers Rust unit/protocol/security tests, real HTTP and private draft saving, actual local Mosquitto TLS positive/negative cases, three browser steps at four widths in two themes, actual Linux virtual CAN discovery/polling in 11/29-bit formats, passive zero-transmit observation, and a full native-process virtual ECU to real broker smoke. Virtual CAN is software acceptance; a physical vehicle, transceiver, proprietary profile and phone-to-pit-wall route still need their own test.

Artifacts: Linux ARM64 and Linux x86-64 archives plus matching GPL-3.0-only corresponding source. The native acquisition path is Linux classical SocketCAN; BLE/serial/TCP ELM adapters belong to G86 Car Computer's integrated service. Existing explicit topic prefixes are preserved; fresh legacy scalar output is optional. Earlier releases retain their original licenses. Third-party dependencies retain their own terms and notices.

All builds and checks are local. The locked dependency audit reports no known RustSec vulnerabilities or warnings as of 2026-10-23. Source and release instructions are included for reproducibility.

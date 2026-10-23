# CAN to MQTT

A local, read-only telemetry bridge for Linux SocketCAN. Version 0.4 adds a guided browser interface, ECU discovery, fresh canonical MQTT packets, and explicit passive signal mappings.

The three setup steps are **Connect → MQTT → Live check**. The live check shows the measurements that actually arrived, their age, the chosen ECU, and broker state. An open interface or saved configuration never counts as verified telemetry.

## Choose the right connection

| Connection | Support |
| --- | --- |
| Linux SocketCAN USB/PCI/HAT interface | This native bridge |
| Standard 11-bit or 29-bit CAN OBD-II | Mode 01 supported-PID discovery and read-only polling |
| A gateway exposing only standard diagnostic PIDs | Only the PIDs that gateway and selected ECU actually expose |
| Passive vehicle broadcast CAN | Engineer-verified signal mappings, or observation of another tool's existing OBD responses |
| Bluetooth LE, serial or TCP ELM327/AT-compatible adapter | G86 Car Computer's integrated adapter service; not a standalone feature of this binary |
| macOS / Windows | The local configuration UI can be built and previewed; SocketCAN acquisition requires Linux |

Standard OBD cannot produce manufacturer-specific race data that the vehicle does not expose. Wheel speeds, brake pressure, steering, gears or proprietary channels need a verified broadcast mapping or a supported ECU bridge. There are no guessed VW or universal race-car profiles.

## Run on Linux

Download the release archive for Linux ARM64 (64-bit Raspberry Pi / ARM computers) or Linux x86-64. The binaries target glibc 2.28 or newer; use a matching modern distribution with the libudev runtime library installed (`libudev1` on Debian/Ubuntu). Extract it and keep the included license and notices with it.

```sh
./can-to-mqtt version
cp example.settings.toml settings.toml
# Edit settings.toml: select your CAN interface, car ID and broker.
./can-to-mqtt --config settings.toml validate-config
./can-to-mqtt --config settings.toml
```

Open **http://127.0.0.1:9093/** on that computer. On first start the service creates a random access token and logs the **file path**, never its value. Read that private file to sign in. The default is `$XDG_STATE_HOME/g86-can-to-mqtt/http-9093.token`, or `$HOME/.config/g86-can-to-mqtt/http-9093.token` when XDG state is unset. The browser keeps the token only in that tab's session storage. Sign out to clear it.

For a headless Pi, forward the local UI through SSH, then open the same URL on your laptop:

```sh
ssh -L 9093:127.0.0.1:9093 your-user@your-pi
```

The setup HTTP service binds only to loopback. It does not expose a plaintext bearer token to the car Wi-Fi. G86 Car Computer provides the integrated HTTPS setup, broker management, downloadable suite, and secure phone QR pairing.

### 1. Connect

Select an interface found by **Find CAN interfaces**. Discovery only reads Linux interface metadata. Bring the interface up with your system's network tools, using the bitrate and wiring specified for your vehicle and adapter. For example, after checking that 500 kbit/s is correct:

```sh
sudo ip link set can0 up type can bitrate 500000
ip -details link show can0
```

Use **Standard OBD** for a diagnostic connection. Auto discovery tries 11-bit then 29-bit CAN, reads supported-PID pages, binds one ECU, then polls only implemented PIDs advertised by that ECU. Standard ECU addresses receive physically addressed requests after binding. An optional response ID filters a known gateway; a custom nonstandard gateway ID keeps functional request addressing because no physical destination can be inferred safely.

Use **Passive** for a connection that must not transmit. Passive mode never opens a transmit socket. It requires explicit verified mappings before it can produce channels. See [passive signals](docs/PASSIVE_SIGNALS.md).

### 2. MQTT

For an integrated installation, let G86 Car Computer configure the private broker. For a standalone installation, supply the broker host, port and credentials. Use TLS with certificate validation for a remote broker; `mqtt_ca_file` can point to a private CA. A private password file avoids putting broker credentials in a command line.

**Validate and save private draft** validates and atomically saves an owner-only JSON file. It does not change running acquisition. Stop the bridge, then run the restart command displayed by the UI. For a systemd installation, copy or promote the reviewed draft to the service's configured file and restart that service. Keep the saved file private: it may contain credentials needed for restart.

### 3. Live check, phone and pit wall

The green check requires fresh source measurements and an actual MQTT connection on this computer. Missing, unsupported, malformed, future-dated, out-of-range or stale measurements are omitted; zero is kept when zero was measured. The UI removes channels after two seconds without a new measurement.

For phone delivery, open G86 Car Computer's **Phone pairing** step and scan its one-use QR in G86 Racing, or paste the pairing code. Check the phone's fresh telemetry indicators. An internet connection and an active pit wall session on the phone are required to forward telemetry to the team. This standalone bridge cannot prove those downstream links.

## Data contract

With `car = "rally-86"`, active OBD packets use `/g86/rally-86/obd/telemetry`. Passive packets use `/g86/rally-86/sensors/<passive_profile>/telemetry`. The profile is one topic segment, matching the phone's sensor subscription.

```json
{
  "schema": 1,
  "source": "obd2/mode01",
  "partial": true,
  "bootId": "random-per-process-identity",
  "sequence": 42,
  "timestampMs": 1790000000123,
  "channels": { "rpm": 2000.0 }
}
```

Each packet contains only values acquired from that response or frame. `timestampMs` is acquisition UTC milliseconds; keep the computer's clock synchronized. The bridge does not repeat cached values with a new time. The sequence increases across interface reconnects within the same boot identity. Canonical live packets use QoS 0 and are not retained. A bounded 64-packet queue drops old or overflowing live packets rather than blocking CAN acquisition; queued packets older than 1.5 seconds are discarded. This is a live stream, not a durable recording journal.

Implemented Mode 01 channels (polled only when advertised by the selected ECU):

| PID | Canonical channel | Unit |
| --- | --- | --- |
| 04 / 05 | engineLoad / coolantC | % / °C |
| 0A / 0B | fuelPressureKpa / manifoldKpa | kPa |
| 0C / 0D | rpm / ecuSpeedKmh | rpm / km/h |
| 0E / 0F | ignitionDeg / intakeC | degrees / °C |
| 10 / 11 | massAirFlowGps / throttle | g/s / % |
| 24 / 2F | lambda / fuelLevel | ratio / % |
| 33 / 42 | baroKpa / batteryV | kPa / V |
| 46 / 5C | ambientC / oilC | °C |

`<base>/health` is retained and contains actual source/broker freshness and clock state. `<base>/status` is retained `online`/`offline` availability with a last will; availability alone is not proof of fresh measurements. Health is updated every five seconds.

For migration, an explicitly configured `mqtt_base_topic` wins over `car`. With neither, the legacy `/GOLF86/ECU/` prefix is preserved. `legacy_scalar_output = true` publishes only the corresponding actually received legacy fields; the old zero-filled 85-field snapshot is no longer emitted. Set it to false for canonical-only installations. Old public library modules remain for source compatibility but are not used by the new acquisition path.

## Configuration and local API

See [example.settings.toml](src/example.settings.toml) for all supported service options. Files can be TOML or JSON. `--config` selects a file; otherwise local `settings.toml` and the standard `/etc/g86-car-telemetry/can-to-mqtt.toml` paths are checked. Explicit `CAN2MQTT_*` environment overrides include `CAN_INTERFACE`, `MQTT_HOST`, `MQTT_PORT`, `MQTT_USERNAME`, `MQTT_PASSWORD`, `MQTT_BASE_TOPIC`, `CAR`, `LOG_LEVEL`, and `HTTP_PORT`.

The local token-protected API provides `GET /api/v1/config` (redacted), `GET /api/v1/status`, `GET /api/v1/discovery`, `GET /api/v1/channels` and `POST /api/v1/config` (draft only). Mutations require `X-G86-Action: configure`; browser origins must match the loopback host. Bodies are bounded to 64 KiB. Static UI assets are embedded, with no CDN, inline script, analytics, or external network dependency.

Optional Prometheus endpoints bind to `127.0.0.1:9090`: `/metrics`, `/health`, `/ready`, `/live`. `/ready` requires fresh actual telemetry, CAN and MQTT. Port 9093 `/healthz` exposes only the readiness boolean without a token. API details require the local token.

`test-connection` opens a receive socket and connects to MQTT without sending diagnostic requests. It verifies those connections only, not ECU data. `--web-only` previews setup without opening a CAN socket. Run under systemd using the hardened [example unit](contrib/can-to-mqtt.service); create a dedicated service user and provision the CAN interface separately.

## Build and verify locally

Rust 1.89 or newer, a C/C++ build toolchain, CMake and pkg-config are required. Linux also needs development headers for libudev (`libudev-dev` on Debian/Ubuntu). MQTT TLS dependencies are built from the locked vendored OpenSSL crate.

```sh
cargo build --release --locked
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo audit
```

See [testing](docs/TESTING.md) for real-process browser checks and isolated Linux virtual CAN tests, and [release instructions](docs/RELEASING.md) for local builds and artifacts. Builds run locally; this repository contains no hosted GitHub build workflows.

## License

First-party source is **GPL-3.0-only** from 0.4.0. See [LICENSE](LICENSE), [licensing notes](LICENSING.md) and [third-party notices](THIRD_PARTY_NOTICES.md). Earlier releases retain the licenses under which they were released.

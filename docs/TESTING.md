# Local verification

Run checks from the repository root:

```sh
cargo fmt --check
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo audit
cargo build --locked
```

The default test suite checks supported-PID scheduling and ECU binding, malformed/truncated responses, canonical units/ranges, provenance, endian/signed mapping, future/stale data, token/Host/Origin policy, response redaction, and a real loopback HTTP save with owner-only draft permissions. No physical CAN device is opened by the default suite.

## Browser acceptance

Install Playwright outside this repository, with Chromium/Chrome available. Node is a development/test tool only and is not shipped with the bridge.

```sh
PLAYWRIGHT_MODULE=/path/to/node_modules/playwright-core \
CHROME_PATH=/path/to/chrome \
CAN_UI_SCREENSHOTS=/tmp/can-ui-proof \
node tests/web-browser.cjs
```

The test starts the actual binary in web-only mode on an ephemeral loopback port, uses a throwaway private config/token, verifies auth/redaction and a private draft save, then injects explicit live/offline status fixtures for UI rendering. It checks all three steps at 320, 390, 768 and 1440 px in dark and light themes, preserves measured zero, removes missing channels and verifies polling does not overwrite unsaved edits. It closes the process and removes its private temporary configuration. `CAN_BINARY` selects another compiled binary.

A separate actual TLS regression uses a throwaway local Mosquitto broker and self-signed certificate with a DNS hostname:

```sh
MOSQUITTO_BIN=/path/to/mosquitto node tests/mqtt-tls.cjs
```

It verifies trusted matching-host success, wrong-host rejection and untrusted-CA rejection. No CAN socket is opened.

## Linux virtual CAN acquisition

Use a newly created, isolated test interface. These tests send synthetic Mode 01 frames and must never be pointed at a real vehicle interface. Creating and removing the interface requires administration; the bridge and tests do not need privileges after the interface is up.

```sh
sudo modprobe vcan
sudo ip link add dev vcanG86 type vcan
sudo ip link set dev vcanG86 up
G86_CAN_TEST_IFACE=vcanG86 cargo test --locked runtime::vcan_tests -- --ignored --test-threads=1 --nocapture
sudo ip link delete dev vcanG86
```

The two opt-in tests exchange actual SocketCAN frames: active supported-PID discovery and physical ECU polling in both 11-bit and 29-bit formats; and verified passive frame decoding with an independent observer proving no CAN transmissions and no cache retiming/republication. Always delete only the temporary interface you created, including after a failed test.

To run a cross-compiled harness on an ARM test host, `cargo zigbuild --release --locked --tests --target aarch64-unknown-linux-gnu.2.28` builds the same tests. Copy the library test executable from the target `release/deps` directory, then run it with `G86_CAN_TEST_IFACE=vcanG86 ... --include-ignored --test-threads=1`. The release instructions explain cross-build native dependencies.

Virtual CAN validates software transport and packet semantics. Physical transceiver wiring, termination, bitrate, a particular vehicle/gateway, actual ELM hardware and phone-to-pit-wall delivery require their own acceptance test.

For a full process-to-broker smoke, build the release binary, keep the isolated vcan interface up, and run:

```sh
G86_CAN_TEST_IFACE=vcanG86 sh tests/vcan-process.sh
```

This starts a temporary local broker, compiles the isolated C ECU fixture, runs the actual bridge and validates canonical RPM publication, fresh readiness and graceful retained offline availability. It requires `cc`, `ip`, `curl`, `mosquitto` and `mosquitto_sub`; its default private test ports are 17983/17993 and may be overridden with `CAN_TEST_MQTT_PORT`/`CAN_TEST_HTTP_PORT`. The fixture refuses physical interfaces. `CAN_BINARY` and `MOSQUITTO_BIN` select executables. The script removes its temporary files and processes, leaving interface cleanup to the person who created it.

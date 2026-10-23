# Local release procedure

First-party source from 0.4.0 is GPL-3.0-only. Distribute the exact corresponding source with binary releases and retain third-party notices. Do not add GitHub Actions or hosted build usage.

1. Start with the reviewed source commit, an up-to-date compatible lockfile and a clean checkout. Confirm Cargo version/license and CHANGELOG.
2. Run the checks in `TESTING.md`, including browser checks for UI changes and isolated virtual CAN checks for acquisition changes. Record real physical acceptance separately when available.
3. Build locally for Linux ARM64 and x86-64 using a target toolchain and C dependency environment. `cargo zigbuild --release --locked --target aarch64-unknown-linux-gnu.2.28` and its `x86_64-unknown-linux-gnu.2.28` counterpart target glibc 2.28. Set `OPENSSL_STATIC=1`; use target-appropriate pkg-config metadata and libudev dependencies. Do not substitute host libudev for a target library.
4. Execute `can-to-mqtt version` and a real source/MQTT smoke check on a matching Linux host. Inspect `file`, ELF dependencies and required GLIBC versions. A cross compile alone is not an execution test.
5. Package `can-to-mqtt`, `example.settings.toml`, `README.md`, `LICENSE`, `LICENSING.md`, `THIRD_PARTY_NOTICES.md`, and `contrib/` in each architecture archive. Exclude private configs, keys, tokens, logs, temporary build directories and unrelated product code.
6. Produce a source archive from the same reviewed commit. Include Cargo.lock, web assets, tests and docs. Record SHA256 sums for every asset and the source commit in the receipt.
7. Only after explicit review/authorization, push the source, tag `v0.4.0` and create the release with matching notes and assets. This document does not itself authorize a push or public release.

Suggested repository description: **Read-only Linux CAN telemetry: guided local setup, standard OBD discovery, verified passive signals, and MQTT for racing.** Suggested topics: `socketcan`, `can-bus`, `obd2`, `mqtt`, `telemetry`, `motorsport`, `rust`, `gpl-3-0`.

The supported release artifacts are Linux ARM64 and Linux x86-64 plus corresponding source. macOS/Windows can compile the web preview/core, while real SocketCAN acquisition is Linux-only; the private G86 Car Computer distribution separately provides cross-platform ELM transports. Do not advertise standalone hardware support merely because a UI/core cross-compile succeeds.

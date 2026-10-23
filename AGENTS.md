# CAN to MQTT maintenance

This repository is the standalone Linux SocketCAN member of the G86 telemetry family. Read `skills/can-telemetry/SKILL.md` when changing acquisition, signal decoding, local setup UI, configuration, or release behavior.

- Public first-party code is GPL-3.0-only. Do not change third-party licenses or assume the G86 product's separate grant covers dependencies.
- Active mode may send only standard OBD Mode 01 read requests. Never add ECU tuning, fault erasure, flashing or arbitrary writes.
- Passive mode must never open a CAN transmit socket. A verified signal definition needs provenance and canonical units; do not invent vehicle mappings.
- Publish actual newly acquired values. Omit unsupported/invalid/stale data; keep a measured zero. Preserve acquisition UTC, boot identity and monotonic sequence across reconnects.
- Keep CAN acquisition independent of broker latency. Queues and retries must remain bounded.
- The UI is embedded plain HTML/CSS/JS with local assets, a strict CSP and textContent-based rendering. Secrets stay out of API responses, URLs, logs, screenshots and Git.
- Configuration saves are private drafts requiring an explicit restart. A save or socket connection must never count as verified source/phone/pit-wall delivery.
- Run locked tests, strict Clippy and an audit for source changes; use the documented real-process browser and virtual CAN checks for these paths. Do not claim physical vehicle testing from a virtual CAN fixture.
- Build releases locally. Never add hosted GitHub builds. A tag or public release needs an explicitly reviewed source commit and matching binaries.

# Changelog

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

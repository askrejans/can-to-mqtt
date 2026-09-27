# can-to-mqtt 0.3.0

Adds non-retained, timestamped G86 packets on `<mqtt_base_topic>/telemetry`.
Existing scalar topics remain. Each canonical packet contains only the supported
Mode 01 PID actually received, with schema 1, source, boot ID, capture timestamp
and sequence. Unsupported/missing PIDs never appear as default zero readings.

Supported canonical channels: RPM, road speed, throttle, engine load, coolant,
intake and oil temperature, fuel/manifold pressure, ignition advance, air flow,
fuel level and battery voltage. Units match the G86 ECU telemetry schema.
Frame validation rejects truncated, negative and unsupported responses.
Existing scalar percentage/pressure arithmetic now avoids byte overflow.

Validation: Linux unit tests and release builds. Actual OBD support, bus bitrate,
response address and polling behavior depend on the vehicle; verify parked.

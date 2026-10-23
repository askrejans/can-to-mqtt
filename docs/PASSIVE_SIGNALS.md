# Verified passive CAN signals

Set `mode = "passive"` to disable all CAN transmissions by this process. No transmitter socket is opened. Raw CAN broadcasting is vehicle-specific: a standard diagnostic gateway often does not expose the vehicle's full internal CAN traffic.

A profile must use real frame IDs, units and scaling verified against an ECU manufacturer's specification or a controlled recording and physical reference. Add a short, specific `source_reference` identifying that evidence and `verified = true`. These fields record the engineer's attestation; the program cannot independently verify a manufacturer specification. No production VW or generic F1/WRC profile is bundled.

```toml
mode = "passive"
car = "lab-car"
passive_profile = "bench-demo"
passive_signals = [
  # SYNTHETIC BENCH EXAMPLE ONLY: not a real vehicle mapping.
  { channel = "rpm", can_id = 291, extended = false, start_bit = 0, bit_length = 16, byte_order = "little", signed = false, factor = 1.0, offset = 0.0, source_reference = "Isolated bench fixture: uint16 little-endian RPM at bytes 0-1", verified = true }
]
```

The example frame `0x123` with bytes `D0 07` yields 2000 rpm. Do not copy this mapping to a real car without verification.

| Field | Meaning |
| --- | --- |
| `channel` | Canonical channel name accepted by the phone; see the names and limits in `telemetry::channel_range` |
| `can_id` / `extended` | Exact 11-bit (0..2047) or 29-bit (0..536870911) identifier and format |
| `start_bit` / `bit_length` | First bit and length, within the received classical CAN data; length 1..32 |
| `byte_order = "little"` | Intel style: bit 0 is the least significant bit of byte 0; significance increases with bit index |
| `byte_order = "big"` | Network order: bit 0 is the most significant bit of byte 0; successive bits form a big-endian integer |
| `signed` | Two's-complement raw value when true |
| `factor` / `offset` | Canonical value = decoded raw × factor + offset |
| `source_reference` / `verified` | Nonempty evidence reference plus explicit verification attestation |

**Big order is not DBC Motorola sawtooth start-bit notation.** Translate and independently test any DBC mapping before using it. Duplicate canonical channels in the same profile, unknown channels, invalid bit ranges, unverified mappings and non-finite scaling are rejected. Truncated frames and out-of-range physical values are omitted.

Each matching received frame produces a partial, nonretained QoS 0 packet at `/g86/<car>/sensors/<profile>/telemetry`, source `sensors/can/<profile>`, with the frame's UTC acquisition time. Profiles contain at most 64 signals; names use a single short lowercase slug. Cached readings are never retransmitted or retimed.

## Observe existing diagnostic responses

`passive_profile = "obd_mode01"` decodes only already-present standard Mode 01 responses from one explicit `obd_response_id`. Set `can_format` if the same numeric ID could occur in both formats. This option does not cause diagnostic polling; another device must already be sending valid requests. Its source is `sensors/can/obd_mode01` on `/g86/<car>/sensors/obd_mode01/telemetry`.

This standalone decoder handles classical CAN single-frame Mode 01 values. It does not claim CAN FD, arbitrary ISO-TP multiframe services, manufacturer UDS decoding, DBC import, ECU commands or durable logging. The integrated G86 Car Computer service separately supports ELM adapters.

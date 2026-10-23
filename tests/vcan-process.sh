#!/bin/sh
# Actual native process -> SocketCAN fixture -> Mosquitto acceptance. Test interface only.
set -eu
: "${G86_CAN_TEST_IFACE:?Set this to an isolated, already-up vcan interface}"
case "$G86_CAN_TEST_IFACE" in vcan*) ;; *) echo 'Use an isolated vcan interface' >&2; exit 2 ;; esac
ip -details -json link show dev "$G86_CAN_TEST_IFACE" | grep -Eq '"info_kind"[[:space:]]*:[[:space:]]*"vcan"' || exit 2
repo=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
can_binary=${CAN_BINARY:-"$repo/target/release/can-to-mqtt"}
mosquitto=${MOSQUITTO_BIN:-mosquitto}
port=${CAN_TEST_MQTT_PORT:-17983}
web_port=${CAN_TEST_HTTP_PORT:-17993}
case "$port:$web_port" in *[!0-9:]*|:*|*:) exit 2 ;; esac
work=$(mktemp -d)
chmod 700 "$work"
bridge= broker= fixture= subscriber=
cleanup() {
    for process in "$bridge" "$fixture" "$subscriber" "$broker"; do
        if [ -n "$process" ]; then kill "$process" 2>/dev/null || true; wait "$process" 2>/dev/null || true; fi
    done
    rm -rf -- "$work"
}
trap cleanup EXIT HUP INT TERM
cc -Wall -Wextra -Werror -O2 "$repo/tests/vcan-ecu.c" -o "$work/vcan-ecu"
printf 'listener %s 127.0.0.1\nallow_anonymous true\npersistence false\n' "$port" > "$work/broker.conf"
# This is a fixed throwaway fixture token on an ephemeral private process, never user credentials.
cat > "$work/smoke.json" <<CONFIG
{"can_interface":"$G86_CAN_TEST_IFACE","can_format":"standard","car":"vcan-proof","mqtt_host":"127.0.0.1","mqtt_port":$port,"legacy_scalar_output":false,"prometheus_enabled":false,"http_port":$web_port,"http_token":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","http_token_file":"$work/token","config_draft_path":"$work/draft.json"}
CONFIG
chmod 600 "$work/smoke.json"
"$mosquitto" -c "$work/broker.conf" > "$work/broker.log" 2>&1 & broker=$!
sleep 0.2
kill -0 "$broker"
mosquitto_sub -h 127.0.0.1 -p "$port" -t /g86/vcan-proof/obd/telemetry -C 1 -W 8 > "$work/packet.json" & subscriber=$!
"$work/vcan-ecu" "$G86_CAN_TEST_IFACE" > "$work/fixture.log" 2>&1 & fixture=$!
"$can_binary" --config "$work/smoke.json" > "$work/bridge.log" 2>&1 & bridge=$!
wait "$subscriber"; subscriber=
grep -F '"rpm":2000.0' "$work/packet.json" >/dev/null
grep -F '"source":"obd2/mode01"' "$work/packet.json" >/dev/null
curl -fsS -H 'Authorization: Bearer aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' "http://127.0.0.1:$web_port/api/v1/status" > "$work/status.json"
grep -F '"ready":true' "$work/status.json" >/dev/null
kill "$bridge"; wait "$bridge"; bridge=
mosquitto_sub -h 127.0.0.1 -p "$port" -t /g86/vcan-proof/obd/status -C 1 -W 3 > "$work/availability"
test "$(cat "$work/availability")" = offline
printf 'PASS: isolated virtual ECU -> actual native bridge -> real MQTT, fresh ready state, graceful offline availability.\n'

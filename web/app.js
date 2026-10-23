"use strict";
const $ = (id) => document.getElementById(id);
let token = sessionStorage.getItem("can-token") || "",
  timer = null,
  current = null;
const theme =
  localStorage.getItem("can-theme") ||
  (matchMedia("(prefers-color-scheme:light)").matches ? "light" : "dark");
document.documentElement.dataset.theme = theme;
$("theme").addEventListener("click", () => {
  const next =
    document.documentElement.dataset.theme === "light" ? "dark" : "light";
  document.documentElement.dataset.theme = next;
  localStorage.setItem("can-theme", next);
});
function error(message) {
  $("error").textContent = message || "";
}
async function api(path, options = {}) {
  const response = await fetch(`api/v1/${path}`, {
    ...options,
    cache: "no-store",
    headers: { Authorization: `Bearer ${token}`, ...options.headers },
  });
  let body;
  try {
    body = await response.json();
  } catch {
    throw Error("The service returned an unreadable response.");
  }
  if (!response.ok) throw Error(body.error || "The request failed.");
  return body;
}
function step(index) {
  document
    .querySelectorAll("[data-panel]")
    .forEach((panel) => (panel.hidden = Number(panel.dataset.panel) !== index));
  document.querySelectorAll("nav button").forEach((button) => {
    if (Number(button.dataset.step) === index)
      button.setAttribute("aria-current", "step");
    else button.removeAttribute("aria-current");
  });
  error("");
}
document
  .querySelectorAll("[data-step]")
  .forEach((button) =>
    button.addEventListener("click", () => step(Number(button.dataset.step))),
  );
document
  .querySelectorAll("[data-next]")
  .forEach((button) =>
    button.addEventListener("click", () => step(Number(button.dataset.next))),
  );
function topics() {
  const car = $("car").value,
    prefix = $("mqtt_base_topic").value.trim() || (car
      ? $("mode").value === "passive"
        ? `/g86/${car}/sensors/${$("passive_profile").value || "can"}`
        : `/g86/${car}/obd`
      : "/telemetry/can");
  $("topics").textContent =
    `Canonical topic: ${prefix}/telemetry. Only actual fresh fields are sent.`;
}
for (const field of ["car", "mqtt_base_topic", "mode", "passive_profile"])
  $(field).addEventListener("input", topics);
async function discover() {
  const data = await api("discovery");
  $("interfaces").replaceChildren(
    ...data.interfaces.map((row) => {
      const option = document.createElement("option");
      option.value = row.name;
      option.textContent = `${row.name} · ${row.state}`;
      return option;
    }),
  );
  $("discovery").textContent = data.interfaces.length
    ? `${data.interfaces.length} CAN interface(s) found. ${data.note}`
    : `No SocketCAN interfaces found on ${data.platform}. ${data.note}`;
}
$("discover").addEventListener("click", () =>
  discover().catch((e) => error(e.message)),
);
function draw(state) {
  $("version").textContent = state.version;
  $("frames").textContent = String(state.frames);
  $("published").textContent = String(state.published);
  $("dropped").textContent = String(state.dropped);
  const ready = state.ready === true;
  $("result").textContent = ready
    ? "Live CAN data and broker publishing verified on this computer. Check your MQTT consumer to verify the next link."
    : state.error ||
      "Waiting for fresh source data and the broker. Saved settings alone do not prove telemetry is arriving.";
  $("result").classList.toggle("good", ready);
  $("link").textContent =
    `CAN: ${state.can_connected ? "connected" : "waiting"} · MQTT: ${state.mqtt_connected ? "connected" : "waiting"} · Mode: ${state.mode}`;
  $("pids").textContent =
    `ECU: ${state.ecu ? `${state.ecu.extended ? "29-bit" : "11-bit"} 0x${state.ecu.id.toString(16).toUpperCase()}` : "not bound"} · Supported standard PIDs: ${(state.supported_pids || []).map((pid) => pid.toString(16).padStart(2, "0").toUpperCase()).join(", ") || "none verified"}`;
  $("channels").replaceChildren(
    ...state.channels.map((channel) => {
      const row = document.createElement("tr");
      for (const value of [
        channel.name,
        Number(channel.value).toLocaleString(undefined, {
          maximumFractionDigits: 3,
        }),
        `${channel.ageMs} ms`,
        channel.source,
      ]) {
        const cell = document.createElement("td");
        cell.textContent = value;
        row.append(cell);
      }
      return row;
    }),
  );
  $("empty").hidden = state.channels.length > 0;
}
async function refresh() {
  try {
    draw(await api("status"));
  } catch (e) {
    error(e.message);
  }
}
async function open() {
  error("");
  current = await api("config");
  for (const field of [
    "can_interface",
    "mode",
    "can_format",
    "passive_profile",
    "car",
    "mqtt_host",
    "mqtt_port",
    "mqtt_username",
    "mqtt_base_topic",
  ])
    $(field).value = current[field] ?? "";
  if (!current.mqtt_base_topic_explicit) $("mqtt_base_topic").value = "";
  $("obd_response_id").value = current.obd_response_id ?? "";
  $("mqtt_tls").value = String(current.mqtt_tls);
  $("passive_signals").value = JSON.stringify(
    current.passive_signals || [],
    null,
    2,
  );
  $("mqtt_password").value = "";
  topics();
  $("access").hidden = true;
  $("product").hidden = false;
  sessionStorage.setItem("can-token", token);
  await discover();
  await refresh();
  clearInterval(timer);
  timer = setInterval(refresh, 500);
}
$("login").addEventListener("submit", (event) => {
  event.preventDefault();
  token = $("token").value.trim();
  open().catch((e) => error(e.message));
});
$("logout").addEventListener("click", () => {
  clearInterval(timer);
  token = "";
  sessionStorage.removeItem("can-token");
  $("token").value = "";
  $("product").hidden = true;
  $("access").hidden = false;
});
$("settings").addEventListener("submit", async (event) => {
  event.preventDefault();
  error("");
  const button = event.submitter;
  button.disabled = true;
  try {
    const body = {};
    for (const field of [
      "can_interface",
      "mode",
      "can_format",
      "passive_profile",
      "car",
      "mqtt_host",
      "mqtt_username",
      "mqtt_base_topic",
    ])
      body[field] = $(field).value.trim();
    body.car = body.car || null;
    body.obd_response_id =
      $("obd_response_id").value === ""
        ? null
        : Number($("obd_response_id").value);
    body.mqtt_port = Number($("mqtt_port").value);
    body.mqtt_tls = $("mqtt_tls").value === "true";
    body.mqtt_password = $("mqtt_password").value;
    body.passive_signals = JSON.parse($("passive_signals").value);
    const result = await api("config", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Bridge-Action": "configure",
      },
      body: JSON.stringify(body),
    });
    $("mqtt_password").value = "";
    $("saved").textContent =
      `Draft saved: ${result.path}\n${result.note}\nRestart command: can-to-mqtt --config ${"\'" + result.path.replaceAll("\'", "\'\\\'\'") + "\'"}`;
  } catch (e) {
    error(e.message);
  } finally {
    button.disabled = false;
  }
});
if (token) open().catch((e) => error(e.message));

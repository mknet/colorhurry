const NUM_LEDS = 10;
const ring = document.getElementById("ring");
const meta = document.getElementById("meta");
const title = document.getElementById("title");
const bleStatus = document.getElementById("ble-status");

const leds = [];
for (let i = 0; i < NUM_LEDS; i++) {
  const led = document.createElement("div");
  led.className = "led";
  const angle = -90 + i * 36;
  led.style.transform = `rotate(${angle}deg) translateY(var(--led-radius))`;
  ring.appendChild(led);
  leds.push(led);
}

let audioCtx = null;

function ensureAudio() {
  if (!audioCtx) {
    audioCtx = new AudioContext();
  }
  if (audioCtx.state === "suspended") {
    audioCtx.resume();
  }
  return audioCtx;
}

function playSequence(steps) {
  if (!steps?.length) return;
  const ctx = ensureAudio();
  let t = ctx.currentTime;
  for (const step of steps) {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.frequency.value = step.hz;
    gain.gain.value = 0.08;
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(t);
    const end = t + step.ms / 1000;
    osc.stop(end);
    t = end;
  }
}

function screenLabel(screen) {
  switch (screen) {
    case "modeSelect":
      return "Mode select";
    case "picker":
      return "Picker";
    case "caller":
      return "Caller";
    default:
      return screen;
  }
}

function paint(view) {
  if (!view) return;
  title.textContent = screenLabel(view.screen);
  const choice = view.modeChoice
    ? ` · next: ${view.modeChoice.role} ch ${view.modeChoice.channel}`
    : "";
  const countdown = view.countdownActive ? ` · beat ${view.countdownStep}` : "";
  meta.textContent = `Channel ${view.channel}${choice}${countdown}`;
  for (let i = 0; i < NUM_LEDS; i++) {
    const c = view.leds?.[i] || { r: 0, g: 0, b: 0 };
    leds[i].style.background = `rgb(${c.r},${c.g},${c.b})`;
  }
}

function sendEvent(event) {
  ensureAudio();
  if (ws?.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({ type: "event", event }));
  }
}

document.getElementById("btn-a").addEventListener("click", () => sendEvent("skip"));
document.getElementById("btn-b").addEventListener("click", () => sendEvent("apply"));
document.getElementById("btn-mode").addEventListener("click", () => sendEvent("reset"));

document.addEventListener("keydown", (e) => {
  if (e.key === "a" || e.key === "ArrowLeft") sendEvent("skip");
  if (e.key === "b" || e.key === "Enter") sendEvent("apply");
  if (e.key === "Escape" || e.key === "m") sendEvent("reset");
});

const proto = location.protocol === "https:" ? "wss:" : "ws:";
let ws = new WebSocket(`${proto}//${location.host}`);

ws.addEventListener("open", () => {
  bleStatus.textContent = "Connected to Node shell";
});

ws.addEventListener("message", (ev) => {
  let msg;
  try {
    msg = JSON.parse(ev.data);
  } catch {
    return;
  }
  if (msg.type === "update") {
    paint(msg.view);
    if (msg.bleStatus) bleStatus.textContent = msg.bleStatus;
    for (const seq of msg.tones || []) {
      playSequence(seq);
    }
  } else if (msg.type === "error") {
    bleStatus.textContent = msg.message || "error";
  }
});

ws.addEventListener("close", () => {
  bleStatus.textContent = "Disconnected — is the server running?";
});

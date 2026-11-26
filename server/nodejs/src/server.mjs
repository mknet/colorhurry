import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { WebSocketServer } from "ws";
import { Game } from "./game.mjs";
import { createBle } from "./ble.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PUBLIC = path.join(__dirname, "..", "public");
const PORT = Number(process.env.PORT || 4747);

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".svg": "image/svg+xml",
  ".ico": "image/x-icon",
};

function sendFile(res, filePath) {
  const ext = path.extname(filePath);
  const type = MIME[ext] || "application/octet-stream";
  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404);
      res.end("Not found");
      return;
    }
    res.writeHead(200, { "Content-Type": type });
    res.end(data);
  });
}

const clients = new Set();

function broadcast(msg) {
  const raw = JSON.stringify(msg);
  for (const ws of clients) {
    if (ws.readyState === 1) ws.send(raw);
  }
}

const game = new Game({
  onUpdate: (msg) => broadcast(msg),
});

const server = http.createServer((req, res) => {
  const url = new URL(req.url || "/", `http://${req.headers.host}`);
  let rel = url.pathname === "/" ? "/index.html" : url.pathname;
  rel = path.normalize(rel).replace(/^(\.\.[/\\])+/, "");
  const filePath = path.join(PUBLIC, rel);
  if (!filePath.startsWith(PUBLIC)) {
    res.writeHead(403);
    res.end("Forbidden");
    return;
  }
  sendFile(res, filePath);
});

const wss = new WebSocketServer({ server });

wss.on("connection", (ws) => {
  clients.add(ws);
  ws.send(JSON.stringify(game.snapshot()));
  ws.on("message", (data) => {
    let msg;
    try {
      msg = JSON.parse(String(data));
    } catch {
      return;
    }
    if (msg?.type === "event" && typeof msg.event === "string") {
      try {
        game.dispatch(msg.event);
      } catch (err) {
        ws.send(JSON.stringify({ type: "error", message: String(err.message || err) }));
      }
    }
  });
  ws.on("close", () => clients.delete(ws));
});

setInterval(() => game.poll(), 10);

const ble = await createBle();
game.attachBle(ble);
game.boot();
game.setBleStatus(
  ble.noble
    ? ble.canAdvertise
      ? "BLE ready (scan + advertise)"
      : `BLE ready (scan). Picker advertise: ${process.platform === "linux" ? "install @abandonware/bleno" : "Linux only"}`
    : "BLE unavailable — UI still drives the core",
);

server.listen(PORT, () => {
  console.log(`Color Hurry Node shell  http://127.0.0.1:${PORT}`);
  console.log(`Core: napi-rs  ·  BLE: ${ble.noble ? "noble" : "off"}  ·  advertise: ${ble.canAdvertise}`);
});

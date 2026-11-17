// Fixture origin on :4001. Serves ../fixtures plus /ws-echo, /sse, /redirect, /form-echo.
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { WebSocketServer } from "ws";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../fixtures");
const PORT = 4001;
const MIME = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8", ".json": "application/json" };

const server = http.createServer((req, res) => {
  const url = new URL(req.url, `http://localhost:${PORT}`);
  if (url.pathname === "/sse") {
    res.writeHead(200, { "content-type": "text/event-stream", "cache-control": "no-store" });
    let i = 0;
    const t = setInterval(() => { res.write(`data: tick-${i++}\n\n`); if (i > 3) { clearInterval(t); res.end(); } }, 50);
    req.on("close", () => clearInterval(t));
    return;
  }
  if (url.pathname === "/redirect") { res.writeHead(302, { location: "/final.html" }); return res.end(); }
  if (url.pathname === "/form-echo" && req.method === "POST") {
    let body = "";
    req.on("data", (c) => (body += c));
    req.on("end", () => { res.writeHead(200, { "content-type": "text/plain; charset=utf-8" }); res.end(`received:${body}`); });
    return;
  }
  const file = path.join(ROOT, url.pathname === "/" ? "health.html" : url.pathname);
  if (!file.startsWith(ROOT)) { res.writeHead(403); return res.end(); }
  fs.readFile(file, (err, data) => {
    if (err) { res.writeHead(404); return res.end("not found"); }
    const headers = { "content-type": MIME[path.extname(file)] ?? "application/octet-stream" };
    if (url.pathname === "/cookie-a.html") headers["set-cookie"] = ["httponly=1; Path=/; HttpOnly", "plain=1; Path=/"];
    res.writeHead(200, headers);
    res.end(data);
  });
});

new WebSocketServer({ server, path: "/ws-echo" }).on("connection", (ws) => ws.on("message", (d) => ws.send(`echo:${d}`)));
server.listen(PORT, "127.0.0.1", () => console.log(`fixture server on http://127.0.0.1:${PORT}`));

// packages/varex-demo/app.js
//
// Demo front end. Two responsibilities:
//   1. Boot the Service Worker (once Phase 1b/4 land; today this logs a
//      clear "not yet registered" message).
//   2. Drive a Wisp handshake against /wisp/ and show it in the log,
//      so you can verify the server is alive without any client build.
//
// It deliberately does not import the full $varex runtime — the runtime
// is what Phase 3 delivers. This page is the smoke test for the server.

const $ = (id) => document.getElementById(id);

const el = {
  dot:        $("dot"),
  statusText: $("status-text"),
  log:        $("log"),
  errorPanel: $("error-panel"),
  errorTitle: $("error-title"),
  errorBody:  $("error-body"),
  url:        $("url"),
  nav:        $("nav"),
  transport:  $("transport"),
};

// Wisp v1.2 packet types and stream types.
const P = {
  CONNECT:  0x01,
  DATA:     0x02,
  CONTINUE: 0x03,
  CLOSE:    0x04,
  INFO:     0x05,
};
const STREAM_TCP = 0x01;

let ws = null;
let nextStreamId = 1;
let reconnectAttempt = 0;
let wispReady = false;

// ---------------------------------------------------------------------------
// UI helpers
// ---------------------------------------------------------------------------

function setStatus(kind, text) {
  el.dot.className = "dot" + (kind === "ok" ? " ok" : kind === "err" ? " err" : "");
  el.statusText.textContent = text;
}

function logLine(kind, msg) {
  const li = document.createElement("li");
  if (kind) li.className = kind;

  const t = document.createElement("span");
  t.className = "t";
  t.textContent = new Date().toLocaleTimeString(undefined, { hour12: false });

  const m = document.createElement("span");
  m.className = "m";
  m.textContent = msg;

  li.append(t, m);
  el.log.prepend(li);

  // Cap the log so a chatty session does not grow the DOM without bound.
  while (el.log.children.length > 250) el.log.lastElementChild.remove();
}

function showError(title, body) {
  el.errorTitle.textContent = title;
  el.errorBody.textContent = body;
  el.errorPanel.hidden = false;
  logLine("err", `${title}: ${body}`);
}

function clearError() {
  el.errorPanel.hidden = true;
}

// ---------------------------------------------------------------------------
// Service Worker registration
// ---------------------------------------------------------------------------

async function registerServiceWorker() {
  if (!("serviceWorker" in navigator)) {
    logLine("warn", "service workers unavailable in this browser");
    return;
  }
  try {
    // /sw.js is emitted by packages/varex-client once Phase 1b wires it.
    // Until then this 404s; we surface that rather than hiding it.
    const reg = await navigator.serviceWorker.register("/sw.js", { scope: "/" });
    logLine("ok", `service worker registered (scope=${reg.scope})`);
  } catch (err) {
    logLine("warn", `service worker not registered: ${err.message ?? err}`);
  }
}

// ---------------------------------------------------------------------------
// Wisp handshake
// ---------------------------------------------------------------------------

function wispUrl() {
  const proto = location.protocol === "https:" ? "wss" : "ws";
  return `${proto}://${location.host}/wisp/`;
}

function connectWisp() {
  const url = wispUrl();
  ws = new WebSocket(url);
  ws.binaryType = "arraybuffer";

  ws.addEventListener("open", () => {
    logLine("ok", `wisp websocket open → ${url}`);

    // Wisp v1.2 §Packet format: type:u8 | stream_id:u32 (LE) | payload.
    // INFO on stream 0 announces our protocol version (2 = v1.2).
    const info = new Uint8Array(6);
    info[0] = P.INFO;
    new DataView(info.buffer).setUint32(1, 0, true);
    info[5] = 0x02;
    ws.send(info);
  });

  ws.addEventListener("message", (ev) => {
    const buf = new Uint8Array(ev.data);
    if (buf.length < 5) return;

    const type = buf[0];
    const streamId = new DataView(buf.buffer).getUint32(1, true);

    if (type === P.CONTINUE) {
      const remaining = new DataView(buf.buffer).getUint32(5, true);
      if (streamId === 0) {
        wispReady = true;
        reconnectAttempt = 0;
        setStatus("ok", "wisp ready");
        logLine("ok", `initial CONTINUE on stream 0, window=${remaining}`);
      } else {
        logLine("", `CONTINUE stream=${streamId} window=${remaining}`);
      }
      return;
    }

    if (type === P.DATA) {
      logLine("", `DATA stream=${streamId} len=${buf.length - 5}`);
      return;
    }

    if (type === P.CLOSE) {
      const reason = buf[5] ?? 0;
      logLine("warn", `CLOSE stream=${streamId} reason=0x${reason.toString(16)}`);
      return;
    }
  });

  ws.addEventListener("close", () => {
    wispReady = false;
    setStatus("err", "wisp disconnected");
    logLine("err", "wisp websocket closed");
    scheduleReconnect();
  });

  ws.addEventListener("error", () => {
    // onclose fires next; keep this quiet to avoid double-logging.
  });
}

function scheduleReconnect() {
  // Exponential backoff with jitter, capped at 30s.
  const base = Math.min(30_000, 250 * 2 ** reconnectAttempt);
  const delay = base + Math.random() * 250;
  reconnectAttempt++;
  logLine("warn", `reconnecting in ${Math.round(delay)}ms`);
  setTimeout(connectWisp, delay);
}

// ---------------------------------------------------------------------------
// Navigation
// ---------------------------------------------------------------------------

function parseTarget(raw) {
  const trimmed = raw.trim();
  if (!trimmed) return null;
  try {
    return new URL(trimmed.includes("://") ? trimmed : `https://${trimmed}`);
  } catch {
    return null;
  }
}

async function navigate(raw) {
  clearError();

  const target = parseTarget(raw);
  if (!target) {
    showError("Invalid URL", raw);
    return;
  }
  if (!/^https?:$/.test(target.protocol)) {
    showError("Unsupported scheme", `${target.protocol} — only http and https are proxied`);
    return;
  }

  logLine("", `navigate → ${target.href}`);

  // The Service Worker owns the actual navigation once it is registered.
  // Check for a controller rather than guessing.
  if (navigator.serviceWorker?.controller) {
    location.assign(`/proxy/${encodeURIComponent(target.href)}`);
    return;
  }

  // No SW yet: demonstrate the Wisp CONNECT so the transport log shows
  // something real. This is the smoke test for Phase 1a, not a page load.
  if (!ws || ws.readyState !== WebSocket.OPEN) {
    showError("Wisp not connected", "The WebSocket to /wisp/ is not open. Check the server logs.");
    return;
  }
  if (!wispReady) {
    showError("Wisp handshake incomplete", "Waiting for the initial CONTINUE on stream 0.");
    return;
  }

  const host = target.hostname;
  const port = Number(target.port || (target.protocol === "https:" ? 443 : 80));
  const streamId = nextStreamId++;

  // CONNECT payload: stream_type:u8 | port:u16 (LE) | hostname (rest, UTF-8).
  const hostBytes = new TextEncoder().encode(host);
  const frame = new Uint8Array(5 + 1 + 2 + hostBytes.length);
  const dv = new DataView(frame.buffer);
  frame[0] = P.CONNECT;
  dv.setUint32(1, streamId, true);
  frame[5] = STREAM_TCP;
  dv.setUint16(6, port, true);
  frame.set(hostBytes, 8);

  ws.send(frame);
  logLine("ok", `CONNECT stream=${streamId} → ${host}:${port}`);

  // If nothing arrives within 5s, say so rather than hanging silently.
  setTimeout(() => {
    logLine("warn", `no response for stream=${streamId} after 5s (expected until Phase 1b)`);
  }, 5000);
}

// ---------------------------------------------------------------------------
// Wiring
// ---------------------------------------------------------------------------

el.nav.addEventListener("submit", (ev) => {
  ev.preventDefault();
  navigate(el.url.value);
});

$("retry").addEventListener("click", () => {
  clearError();
  navigate(el.url.value);
});

$("back").addEventListener("click", () => {
  clearError();
});

$("clear-log").addEventListener("click", () => {
  el.log.replaceChildren();
});

el.transport.addEventListener("change", (ev) => {
  logLine("", `transport preference = ${ev.target.value} (applied by the SW once registered)`);
});

// Boot.
setStatus("ok", "booting");
registerServiceWorker();
connectWisp();

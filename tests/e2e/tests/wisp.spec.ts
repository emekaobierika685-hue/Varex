// Protocol-level tests through a real browser WebSocket.
import { test, expect, wispConnectReason } from "../helpers.js";

test("initial CONTINUE arrives on stream 0 with the configured window", async ({ page, baseURL }) => {
  await page.goto(baseURL! + "/healthz");
  const window = await page.evaluate(
    () =>
      new Promise<number>((resolve, reject) => {
        const ws = new WebSocket(`ws://${location.host}/wisp/`);
        ws.binaryType = "arraybuffer";
        const t = setTimeout(() => reject(new Error("timeout")), 5000);
        ws.onmessage = (ev) => {
          const b = new Uint8Array(ev.data as ArrayBuffer);
          if (b.length < 9 || b[0] !== 0x03) return;
          const dv = new DataView(b.buffer);
          if (dv.getUint32(1, true) !== 0) return;
          clearTimeout(t); ws.close(); resolve(dv.getUint32(5, true));
        };
        ws.onerror = () => reject(new Error("ws error"));
      }),
  );
  expect(window).toBe(4); // varex.test.toml: credit_window = 4
});

test("a malformed CONNECT does not kill the connection", async ({ page, baseURL }) => {
  await page.goto(baseURL! + "/healthz");
  // Invalid stream_type: the server drops the packet (no CLOSE for it)...
  const noClose = await wispConnectReason(page, { host: "example.com", port: 443, streamType: 0x99, waitMs: 1200 });
  expect(noClose).toBe(-1);
});

test("a fresh connection still works after a malformed packet", async ({ page, baseURL }) => {
  await page.goto(baseURL! + "/healthz");
  // ...and a policy-violating CONNECT on a new socket gets a CLOSE.
  expect(await wispConnectReason(page, { host: "10.0.0.1", port: 80 })).not.toBe(-1);
});

// Shared fixtures. `proxyReady` is false unless VAREX_E2E_FULL=1 AND the
// /proxy/ route exists. Gated tests call:
//   test.skip(!proxyReady, "requires Phase 1b+ ...")
import { test as base, expect } from "@playwright/test";

// Two different origins pointing at the same fixture server.
export const ORIGIN_A = process.env.VAREX_FIXTURE_A ?? "http://127.0.0.1:4001";
export const ORIGIN_B = process.env.VAREX_FIXTURE_B ?? "http://localhost:4001";

export const proxied = (url: string) => `/proxy/${encodeURIComponent(url)}`;

export const test = base.extend<{ proxyReady: boolean }>({
  proxyReady: async ({ request }, use) => {
    if (process.env.VAREX_E2E_FULL !== "1") return use(false);
    const probe = await request.get(proxied(`${ORIGIN_A}/health.html`), { maxRedirects: 0 });
    await use(probe.status() !== 404 && probe.status() !== 400);
  },
});

export { expect };

/** Open a Wisp socket, wait for the initial CONTINUE, send a CONNECT, return the CLOSE reason (or -1 if none within `waitMs`). */
export async function wispConnectReason(
  page: import("@playwright/test").Page,
  opts: { host: string; port: number; streamType?: number; waitMs?: number },
): Promise<number> {
  return page.evaluate(
    ({ host, port, streamType, waitMs }) =>
      new Promise<number>((resolve, reject) => {
        const ws = new WebSocket(`ws://${location.host}/wisp/`);
        ws.binaryType = "arraybuffer";
        const fail = setTimeout(() => reject(new Error("timeout")), 10000);
        let sent = false;
        ws.onmessage = (ev) => {
          const b = new Uint8Array(ev.data as ArrayBuffer);
          if (b.length < 5) return;
          const dv = new DataView(b.buffer);
          const id = dv.getUint32(1, true);
          if (b[0] === 0x03 && id === 0 && !sent) {
            sent = true;
            const hb = new TextEncoder().encode(host);
            const f = new Uint8Array(8 + hb.length);
            const fd = new DataView(f.buffer);
            f[0] = 0x01; fd.setUint32(1, 1, true); f[5] = streamType; fd.setUint16(6, port, true); f.set(hb, 8);
            ws.send(f);
            if (waitMs > 0) setTimeout(() => { clearTimeout(fail); ws.close(); resolve(-1); }, waitMs);
            return;
          }
          if (b[0] === 0x04 && id === 1) { clearTimeout(fail); ws.close(); resolve(b[5]); }
        };
        ws.onerror = () => reject(new Error("ws error"));
      }),
    { host: opts.host, port: opts.port, streamType: opts.streamType ?? 1, waitMs: opts.waitMs ?? 0 },
  );
}

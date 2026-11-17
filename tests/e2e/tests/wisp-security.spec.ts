// SSRF regression tests. They need no outbound network: every target is
// refused before any connection attempt. A failure here is a security bug.
// NOTE: 0x03 is the provisional "refused" code used by the server; if
// protocol.rs reason codes change to match the spec table, update REFUSED.
import { test, expect, wispConnectReason } from "../helpers.js";

const REFUSED = 0x03;
const cases: [string, string, number][] = [
  ["loopback", "127.0.0.1", 80],
  ["private 10/8", "10.0.0.1", 80],
  ["private 192.168/16", "192.168.1.1", 80],
  ["private 172.16/12", "172.16.0.1", 80],
  ["link-local", "169.254.169.254", 80],
  ["carrier-grade NAT", "100.64.0.1", 80],
  ["denied hostname", "localhost", 80],
  ["denied suffix", "service.internal", 80],
  ["disallowed port", "example.com", 6379],
];

for (const [name, host, port] of cases) {
  test(`CONNECT to ${name} (${host}:${port}) is refused`, async ({ page, baseURL }) => {
    await page.goto(baseURL! + "/healthz");
    expect(await wispConnectReason(page, { host, port })).toBe(REFUSED);
  });
}

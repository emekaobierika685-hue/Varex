// Gated: needs Phase 3 (runtime WebSocket hook). Runs only with VAREX_E2E_FULL=1 once the proxy route exists.
// NOTE: fixture origins are on loopback, which the SSRF guard blocks. Full mode
// requires a test-only flag to allow loopback (not implemented yet; see docs/CONTRIBUTING.md).
import { test, expect, ORIGIN_A, ORIGIN_B, proxied } from "../helpers.js";
const SKIP = "requires Phase 3 (runtime WebSocket hook)";

test("WebSocket echo works through the proxy", async ({ page, proxyReady }) => {
  test.skip(!proxyReady, SKIP);
  await page.goto(proxied(`${ORIGIN_A}/ws.html`));
  await page.click("#go");
  await expect(page.locator("#result")).toHaveText("echo:ping", { timeout: 15000 });
});

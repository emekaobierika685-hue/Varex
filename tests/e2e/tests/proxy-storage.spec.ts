// Gated: needs Phase 5 (origin isolation). Runs only with VAREX_E2E_FULL=1 once the proxy route exists.
// NOTE: fixture origins are on loopback, which the SSRF guard blocks. Full mode
// requires a test-only flag to allow loopback (not implemented yet; see docs/CONTRIBUTING.md).
import { test, expect, ORIGIN_A, ORIGIN_B, proxied } from "../helpers.js";
const SKIP = "requires Phase 5 (origin isolation)";

test("localStorage is isolated between origins", async ({ page, proxyReady }) => {
  test.skip(!proxyReady, SKIP);
  await page.goto(proxied(`${ORIGIN_A}/storage.html`));
  await expect(page.locator("#result")).toHaveText("wrote", { timeout: 15000 });
  await page.goto(proxied(`${ORIGIN_B}/storage.html?read=1`));
  await expect(page.locator("#result")).toHaveText("empty", { timeout: 15000 });
});

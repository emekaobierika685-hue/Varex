// Gated: needs Phase 1b + 4 (transport, SW routing). Runs only with VAREX_E2E_FULL=1 once the proxy route exists.
// NOTE: fixture origins are on loopback, which the SSRF guard blocks. Full mode
// requires a test-only flag to allow loopback (not implemented yet; see docs/CONTRIBUTING.md).
import { test, expect, ORIGIN_A, ORIGIN_B, proxied } from "../helpers.js";
const SKIP = "requires Phase 1b + 4 (transport, SW routing)";

test("child iframe loads through the proxy", async ({ page, proxyReady }) => {
  test.skip(!proxyReady, SKIP);
  await page.goto(proxied(`${ORIGIN_A}/iframe.html`));
  await expect(page.frameLocator("iframe").locator("h1")).toHaveText("inner", { timeout: 15000 });
});

test("runtime is active inside the child frame", async ({ page, proxyReady }) => {
  test.skip(!proxyReady, SKIP);
  await page.goto(proxied(`${ORIGIN_A}/iframe.html`));
  await expect(page.frameLocator("iframe").locator("#runtime-marker")).toHaveText("active", { timeout: 15000 });
});

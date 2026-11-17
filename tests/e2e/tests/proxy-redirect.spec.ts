// Gated: needs Phase 4 (redirect rewriting). Runs only with VAREX_E2E_FULL=1 once the proxy route exists.
// NOTE: fixture origins are on loopback, which the SSRF guard blocks. Full mode
// requires a test-only flag to allow loopback (not implemented yet; see docs/CONTRIBUTING.md).
import { test, expect, ORIGIN_A, ORIGIN_B, proxied } from "../helpers.js";
const SKIP = "requires Phase 4 (redirect rewriting)";

test("302 redirect stays inside the proxy", async ({ page, proxyReady }) => {
  test.skip(!proxyReady, SKIP);
  await page.goto(proxied(`${ORIGIN_A}/redirect`));
  await expect(page.locator("h1")).toHaveText("final", { timeout: 15000 });
  expect(page.url()).toContain("/proxy/");
});

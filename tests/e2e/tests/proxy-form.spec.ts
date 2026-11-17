// Gated: needs Phase 3 (form action rewriting). Runs only with VAREX_E2E_FULL=1 once the proxy route exists.
// NOTE: fixture origins are on loopback, which the SSRF guard blocks. Full mode
// requires a test-only flag to allow loopback (not implemented yet; see docs/CONTRIBUTING.md).
import { test, expect, ORIGIN_A, ORIGIN_B, proxied } from "../helpers.js";
const SKIP = "requires Phase 3 (form action rewriting)";

test("POST form submits through the proxy", async ({ page, proxyReady }) => {
  test.skip(!proxyReady, SKIP);
  await page.goto(proxied(`${ORIGIN_A}/form.html`));
  await page.fill("input[name=q]", "hello");
  await page.click("button[type=submit]");
  await expect(page.locator("body")).toContainText("received:q=hello", { timeout: 15000 });
});

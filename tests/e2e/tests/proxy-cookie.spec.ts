// Gated: needs Phase 4 (cookie jar). Runs only with VAREX_E2E_FULL=1 once the proxy route exists.
// NOTE: fixture origins are on loopback, which the SSRF guard blocks. Full mode
// requires a test-only flag to allow loopback (not implemented yet; see docs/CONTRIBUTING.md).
import { test, expect, ORIGIN_A, ORIGIN_B, proxied } from "../helpers.js";
const SKIP = "requires Phase 4 (cookie jar)";

test("cookies are isolated between origins", async ({ page, proxyReady }) => {
  test.skip(!proxyReady, SKIP);
  await page.goto(proxied(`${ORIGIN_A}/cookie-a.html`));
  await page.evaluate(() => (document.cookie = "session=alpha; path=/"));
  await page.goto(proxied(`${ORIGIN_B}/cookie-b.html`));
  expect(await page.evaluate(() => document.cookie)).not.toContain("session=alpha");
});

test("HttpOnly cookies are hidden from document.cookie", async ({ page, proxyReady }) => {
  test.skip(!proxyReady, SKIP);
  await page.goto(proxied(`${ORIGIN_A}/cookie-a.html`));
  const seen = await page.evaluate(() => document.cookie);
  expect(seen).toContain("plain=1");
  expect(seen).not.toContain("httponly=1");
});

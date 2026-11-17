// Starts two servers from the repo root:
//   1. varex-server on :4000 using tests/e2e/varex.test.toml
//   2. fixture server on :4001 (static fixtures + /ws-echo, /sse, /redirect, /form-echo)
// Tests that need the Service Worker, rewriter, runtime or cookie jar are
// skipped unless VAREX_E2E_FULL=1 (see helpers.ts).
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  timeout: 30_000,
  expect: { timeout: 10_000 },
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: process.env.VAREX_BASE_URL ?? "http://127.0.0.1:4000",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: process.env.VAREX_NO_WEBSERVER
    ? undefined
    : [
        {
          command: "cargo run --quiet --release -p varex-wisp --bin varex-server -- --config tests/e2e/varex.test.toml",
          cwd: "../..",
          url: "http://127.0.0.1:4000/healthz",
          reuseExistingServer: true,
          timeout: 240_000,
        },
        {
          command: "node tests/e2e/fixture-server.mjs",
          cwd: "../..",
          url: "http://127.0.0.1:4001/health.html",
          reuseExistingServer: true,
          timeout: 30_000,
        },
      ],
});

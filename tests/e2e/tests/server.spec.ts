// Always-run control-plane tests.
import { test, expect } from "../helpers.js";

test("healthz returns ok", async ({ request }) => {
  const r = await request.get("/healthz");
  expect(r.status()).toBe(200);
  expect((await r.text()).trim()).toBe("ok");
});

test("readyz returns ready under capacity", async ({ request }) => {
  const r = await request.get("/readyz");
  expect(r.status()).toBe(200);
  expect((await r.text()).trim()).toBe("ready");
});

test("metrics are Prometheus text", async ({ request }) => {
  const r = await request.get("/metrics");
  expect(r.status()).toBe(200);
  expect(await r.text()).toMatch(/^# (HELP|TYPE) /m);
});

test("unknown route is 404", async ({ request }) => {
  expect((await request.get("/no-such-route")).status()).toBe(404);
});

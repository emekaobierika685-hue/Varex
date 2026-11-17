// crates/varex-core/src/metrics_defs.rs
//! Metric name constants and helpers. Named `metrics_defs` because
//! `pub mod metrics` shadowed the external `metrics` crate inside
//! `varex-core`, which made `metrics::set_global_recorder` in `main.rs`
//! resolve to the wrong module in the previous draft.
use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};
use std::time::Duration;

pub const CONNECTIONS_ACTIVE: &str = "varex_connections_active";
pub const STREAMS_ACTIVE: &str = "varex_streams_active";
pub const BYTES_OUT: &str = "varex_bytes_out_total";
pub const BYTES_IN: &str = "varex_bytes_in_total";
pub const CONNECT_LATENCY: &str = "varex_connect_seconds";
pub const CLOSE_REASON: &str = "varex_close_total";
pub const RATE_LIMITED: &str = "varex_rate_limited_total";
pub const SSRF_BLOCKED: &str = "varex_ssrf_blocked_total";

pub fn describe_all() {
    describe_gauge!(CONNECTIONS_ACTIVE, "Active WebSocket connections");
    describe_gauge!(STREAMS_ACTIVE, "Active multiplexed streams");
    describe_counter!(BYTES_OUT, "Bytes sent to clients");
    describe_counter!(BYTES_IN, "Bytes received from clients");
    describe_histogram!(CONNECT_LATENCY, "Upstream connect latency");
    describe_counter!(CLOSE_REASON, "Streams closed by reason");
    describe_counter!(RATE_LIMITED, "Rate limited");
    describe_counter!(SSRF_BLOCKED, "SSRF blocked");
}
pub fn conn_open() { gauge!(CONNECTIONS_ACTIVE).increment(1.0); }
pub fn conn_closed() { gauge!(CONNECTIONS_ACTIVE).decrement(1.0); }
pub fn stream_open() { gauge!(STREAMS_ACTIVE).increment(1.0); }
pub fn stream_closed() { gauge!(STREAMS_ACTIVE).decrement(1.0); }
pub fn bytes_out(n: u64) { counter!(BYTES_OUT).increment(n); }
pub fn bytes_in(n: u64) { counter!(BYTES_IN).increment(n); }
pub fn connect_latency(d: Duration) { histogram!(CONNECT_LATENCY).record(d.as_secs_f64()); }
pub fn close(reason: u8) { counter!(CLOSE_REASON, "reason" => reason.to_string()).increment(1); }
pub fn rate_limited() { counter!(RATE_LIMITED).increment(1); }
pub fn ssrf_blocked() { counter!(SSRF_BLOCKED).increment(1); }

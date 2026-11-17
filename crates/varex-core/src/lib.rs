// crates/varex-core/src/lib.rs
#![forbid(unsafe_code)]
//! Shared config, errors, SSRF policy, rate limiter, DNS cache, metrics
//! names, and the plugin trait. The metrics module is named `metrics_defs`
//! to avoid shadowing the external `metrics` crate, which caused
//! `metrics::set_global_recorder` in `main.rs` to resolve to the local
//! module in the previous draft.

pub mod config;
pub mod dnscache;
pub mod error;
pub mod metrics_defs;
pub mod plugin;
pub mod ratelimit;
pub mod ssrf;

pub use config::Config;
pub use error::{Result, VarexError};
pub use ssrf::SsrfPolicy;

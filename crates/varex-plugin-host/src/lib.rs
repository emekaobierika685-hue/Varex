// crates/varex-plugin-host/src/lib.rs
#![forbid(unsafe_code)]
//! Phase 0 ships the host trait implementation skeleton; the plugin
//! loader is wired in Phase 5 once the sandbox policy (fuel, memory,
//! wall-clock) is finalized.
pub mod example_blocklist;

pub use example_blocklist::BlocklistPlugin;

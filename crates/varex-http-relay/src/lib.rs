// crates/varex-http-relay/src/lib.rs
#![forbid(unsafe_code)]
pub mod client;
pub mod handler;
pub use client::UpstreamClient;
pub use handler::{relay_handler, RelayState};

use std::sync::Arc;
use axum::Router;

pub fn router(state: Arc<RelayState>) -> Router {
    use axum::routing::any;
    Router::new().route("/relay", any(relay_handler)).with_state(state)
}
